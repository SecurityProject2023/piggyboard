use actix_csrf::extractor::{CsrfToken, CsrfGuarded, Csrf};
use actix_identity::Identity;
use actix_web::{web::{self, Data, Form}, get, post, HttpResponse, Responder};
use diesel::{r2d2::{PooledConnection, self}, SqliteConnection};
use piggyboard::{models::{User, Article, Like, Dislike}, DbPool, cryp::md5};
use serde_json::json;
use tera::{Tera, Context};
use serde::Deserialize;

#[get("/")]
async fn index(pool: Data<DbPool>, /* user: Option<Identity>,*/ tera: web::Data<Tera>) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let articles = Article::get_all(&mut conn);
  if let Err(_) = articles { return HttpResponse::InternalServerError().finish(); }
  let articles: Vec<(usize, Article)> = articles.unwrap().into_iter().enumerate().collect::<Vec<(usize, Article)>>();
  let authors = articles.iter().map(|a| User::by_id(&mut conn, a.1.author_id).unwrap()).collect::<Vec<User>>();
  let mut ctx = Context::new();
  ctx.insert("articles", &articles);
  ctx.insert("authors", &authors);
  let result = tera.render("index.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[get("/write")]
async fn gwrite(token: CsrfToken, user: Option<Identity>, tera: web::Data<Tera>) -> impl Responder {
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let mut ctx = Context::new();
  ctx.insert("csrf_token", &token.get());
  let result = tera.render("write.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/write")]
async fn pwrite(pool: Data<DbPool>, user: Option<Identity>, data: Csrf<Form<ArticleWriteData>>) -> impl Responder {
  let data = data.into_inner().into_inner();
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let user = User::by_username(&mut conn, &user.unwrap().id().unwrap());
  if let Err(_) = user { return HttpResponse::Unauthorized().finish(); }
  let user = user.unwrap();
  Article::add(&mut conn, Article::new(user.id, data.title, data.content)).unwrap();
  HttpResponse::Found().append_header(("Location", "/")).finish()
}

#[get("/article/{id}")]
async fn garticle(pool: Data<DbPool>, token: CsrfToken, user: Option<Identity>, tera: web::Data<Tera>, ath: web::Path<(u32,)>) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let mut ctx = Context::new();
  if let Some(user) = user {
    if let Ok(user) = User::by_username(&mut conn, &user.id().unwrap()) {
      ctx.insert("user", &user);
    }
  }
  let arti = Article::by_id(&mut conn, ath.into_inner().0 as i32);
  if let Err(_) = arti { return HttpResponse::NotFound().finish(); }
  let arti = arti.unwrap();
  let author = arti.author(&mut conn).unwrap();
  ctx.insert("article", &arti);
  ctx.insert("author", &author);
  ctx.insert("amd5", &md5(&author.email));
  ctx.insert("csrf_token", &token.get());
  ctx.insert("likes", &Like::get_all(&mut conn, arti.id).unwrap());
  ctx.insert("dislikes", &Dislike::get_all(&mut conn, arti.id).unwrap());
  let result = tera.render("view.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/api/v1/rate/{id}")]
async fn prate_article(pool: Data<DbPool>, user: Option<Identity>, ath: web::Path<(u32,)>, payload: web::Json<ArticleRateData>) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let user = User::by_username(&mut conn, &user.unwrap().id().unwrap());
  if let Err(_) = user { return HttpResponse::Unauthorized().finish(); }
  let user = user.unwrap();
  let arti = Article::by_id(&mut conn, ath.into_inner().0 as i32);
  if let Err(_) = arti { return HttpResponse::NotFound().finish(); }
  let arti: Article = arti.unwrap();
  let payload = payload.into_inner();
  if user.id != payload.user_id { return HttpResponse::Unauthorized().finish(); }
  if payload.value == 1 {
    if let Ok(true) = Like::liked(&mut conn, user.id, arti.id) { return HttpResponse::Ok().json(json!({ "error": true, "message": "You Already Liked This" })) }
    Like::add(&mut conn, Like::new(payload.user_id, arti.id)).unwrap();
  } else if payload.value == -1 {
    if let Ok(true) = Dislike::disliked(&mut conn, user.id, arti.id) { return HttpResponse::Ok().json(json!({ "error": true, "message": "You Already Disliked This" })) }
    Dislike::add(&mut conn, Dislike::new(payload.user_id, arti.id)).unwrap();
  } else { return HttpResponse::BadRequest().finish(); }
  HttpResponse::Ok().json(json!({ "error": false, "message": "Success" }))
}

#[get("/moonrin")]
async fn moonrin() -> impl Responder { HttpResponse::InternalServerError().finish() }

#[derive(Deserialize)]
pub struct ArticleWriteData {
  csrf_token: CsrfToken,
  pub title: String,
  pub content: String,
}

impl CsrfGuarded for ArticleWriteData { fn csrf_token(&self) -> &CsrfToken { &self.csrf_token } }

#[derive(Deserialize)]
pub struct ArticleRateData {
  pub value: i32,
  pub user_id: i32,
  pub csrf_token: String
}

pub fn services(cfg: &mut web::ServiceConfig) {
  cfg.service(index);
  cfg.service(moonrin);
  cfg.service(garticle);
  cfg.service(gwrite);
  cfg.service(pwrite);
  cfg.service(prate_article);
}