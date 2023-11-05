use actix_csrf::extractor::{CsrfToken, CsrfGuarded};
use actix_identity::Identity;
use actix_web::{web::{self, Data, Form}, get, post, HttpResponse, Responder};
use diesel::{r2d2::{PooledConnection, self}, SqliteConnection};
use piggyboard::{models::{User, Article, Like, Dislike, Comment}, DbPool, cryp::md5};
use tera::{Tera, Context};
use serde::Deserialize;

#[get("/")]
async fn index(pool: Data<DbPool>, user: Option<Identity>, tera: web::Data<Tera>) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let articles = Article::get_all(&mut conn);
  if let Err(_) = articles { return HttpResponse::InternalServerError().finish(); }
  let articles: Vec<(usize, Article)> = articles.unwrap().into_iter().enumerate().collect::<Vec<(usize, Article)>>();
  let authors = articles.iter().map(|a| User::by_id(&mut conn, a.1.author_id).unwrap()).collect::<Vec<User>>();
  let mut ctx = Context::new();
  if let Some(user) = user {
    if let Ok(user) = User::by_username(&mut conn, &user.id().unwrap()) {
      ctx.insert("user", &user);
    }
  }
  ctx.insert("articles", &articles);
  ctx.insert("authors", &authors);
  let result = tera.render("index.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[get("/updates")]
async fn gupdates(pool: Data<DbPool>, user: Option<Identity>, tera: web::Data<Tera>) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let mut ctx = Context::new();
  if let Some(user) = user {
    if let Ok(user) = User::by_username(&mut conn, &user.id().unwrap()) {
      ctx.insert("user", &user);
      ctx.insert("amd5", &md5(&user.email));
    }
  }
  let result = tera.render("updates.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[get("/write")]
async fn gwrite(pool: Data<DbPool>, token: CsrfToken, user: Option<Identity>, tera: web::Data<Tera>) -> impl Responder {
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let mut ctx = Context::new();
  if let Some(user) = user {
    if let Ok(user) = User::by_username(&mut conn, &user.id().unwrap()) {
      ctx.insert("user", &user);
    } else { return HttpResponse::Unauthorized().finish(); }
  }
  ctx.insert("csrf_token", &token.get());
  let result = tera.render("write.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/write")]
async fn pwrite(pool: Data<DbPool>, user: Option<Identity>, data: Form<ArticleWriteData>) -> impl Responder {
  let data = data.into_inner();
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
  let comment = arti.get_all_comments(&mut conn).unwrap();
  let comment_authors = arti.get_all_comment_authors(&mut conn).unwrap();
  ctx.insert("article", &arti);
  ctx.insert("author", &author);
  ctx.insert("amd5", &md5(&author.email));
  ctx.insert("csrf_token", &token.get());
  ctx.insert("likes", &Like::get_all(&mut conn, arti.id).unwrap());
  ctx.insert("dislikes", &Dislike::get_all(&mut conn, arti.id).unwrap());
  ctx.insert("comments", &comment);
  ctx.insert("comment_authors", &comment_authors);
  let result = tera.render("view.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/article/{id}/comment")]
async fn pcomment(pool: Data<DbPool>, user: Option<Identity>, data: Form<CommentWriteData>, ath: web::Path<(u32,)>) -> impl Responder {
  let data = data.into_inner();
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let user = User::by_username(&mut conn, &user.unwrap().id().unwrap());
  if let Err(_) = user { return HttpResponse::Unauthorized().finish(); }
  let user = user.unwrap();
  let arti = Article::by_id(&mut conn, ath.into_inner().0 as i32);
  if let Err(_) = arti { return HttpResponse::NotFound().finish(); }
  let arti = arti.unwrap();
  Comment::add(&mut conn, Comment::new(arti.id, user.id, data.content)).unwrap();
  HttpResponse::Found().append_header(("Location", format!("/article/{}", arti.id))).finish()
}

#[get("/moonrin")]
async fn moonrin() -> impl Responder { HttpResponse::InternalServerError().finish() }

#[derive(Deserialize)]
pub struct ArticleWriteData {
  csrf_token: CsrfToken,
  pub title: String,
  pub content: String,
}

#[derive(Deserialize)]
pub struct CommentWriteData {
  csrf_token: CsrfToken,
  pub content: String,
}

impl CsrfGuarded for ArticleWriteData { fn csrf_token(&self) -> &CsrfToken { &self.csrf_token } }
impl CsrfGuarded for CommentWriteData { fn csrf_token(&self) -> &CsrfToken { &self.csrf_token } }

pub fn services(cfg: &mut web::ServiceConfig) {
  cfg.service(index);
  cfg.service(moonrin);
  cfg.service(garticle);
  cfg.service(gwrite);
  cfg.service(pwrite);
  cfg.service(gupdates);
  cfg.service(pcomment);
}