use std::collections::HashMap;

use actix_csrf::extractor::{CsrfToken, CsrfGuarded};
use actix_identity::Identity;
use actix_web::{web::{self, Data, Form}, get, post, HttpResponse, Responder};
use diesel::{r2d2::{PooledConnection, self}, SqliteConnection};
use piggyboard::{models::{User, Article, Like, Dislike, Comment, ArticleAcl, AclData}, DbPool, cryp::md5, PiggyResult, utils::Acl};
use tera::{Tera, Context};
use serde::Deserialize;

#[get("/")]
async fn index(pool: Data<DbPool>, user: Option<Identity>, tera: web::Data<Tera>) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let articles = Article::get_all(&mut conn);
  if let Err(_) = articles { return HttpResponse::InternalServerError().finish(); }
  let articles: Vec<(usize, Article)> = articles.unwrap().into_iter().enumerate().collect::<Vec<(usize, Article)>>();
  let authors = articles.iter().map(|a| User::by_id(&mut conn, a.1.author_id).unwrap()).collect::<Vec<User>>();
  let acls = articles.iter().map(|a| a.1.acl(&mut conn).unwrap()).collect::<Vec<ArticleAcl>>();
  let mut ctx = Context::new();
  if let Some(user) = user {
    if let Ok(user) = User::by_username(&mut conn, &user.id().unwrap()) {
      ctx.insert("user", &user);
      ctx.insert("uacl", &(user.acl() as i32));
    }
  } else { ctx.insert("uacl", &(Acl::All as i32)); }
  ctx.insert("articles", &articles);
  ctx.insert("authors", &authors);
  ctx.insert("acls", &acls);
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

#[get("/u/@{username}")]
async fn guser(pool: Data<DbPool>, user: Option<Identity>, tera: web::Data<Tera>, username: web::Path<(String,)>) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let mut ctx = Context::new();
  match User::by_username(&mut conn, &username.into_inner().0) {
    Ok(r) => {
      ctx.insert("user", &r);
      ctx.insert("amd5", &md5(&r.email));
      let articles = Article::by_author_id(&mut conn, r.id).unwrap();
      let comments = Comment::by_author_id(&mut conn, r.id).unwrap();
      ctx.insert("articles", &articles);
      ctx.insert("comments", &comments);
      let mut acls: HashMap<i32, ArticleAcl> = articles.iter().map(|a| (a.id, a.acl(&mut conn).unwrap())).collect::<HashMap<i32, ArticleAcl>>();
      for c in &comments {
        if let None = acls.get(&c.article_id) { acls.insert(c.article_id, ArticleAcl::by_article_id(&mut conn, c.article_id).unwrap()); }
      }
      ctx.insert("acls", &acls);
    },
    Err(_) => return HttpResponse::NotFound().finish(),
  };
  if let Some(user) = user {
    if let Ok(user) = User::by_username(&mut conn, &user.id().unwrap()) {
      ctx.insert("login", &user);
      ctx.insert("lamd5", &md5(&user.email));
      ctx.insert("uacl", &(user.acl() as i32));
    }
  } else { ctx.insert("uacl", &(Acl::All as i32)); }
  let result = tera.render("users.html", &ctx).unwrap();
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
async fn pwrite(pool: Data<DbPool>, user: Option<Identity>, data: Form<ArticleWriteData>, tera: web::Data<Tera>) -> impl Responder {
  let data = data.into_inner();
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let user = User::by_username(&mut conn, &user.unwrap().id().unwrap());
  if let Err(_) = user { return HttpResponse::Unauthorized().finish(); }
  if data.title.is_empty() || data.content.is_empty() {
    let mut ctx = Context::new();
    ctx.insert("error", "Title or content is empty.");
    ctx.insert("csrf_token", &data.csrf_token);
    return HttpResponse::Ok().body(tera.render("write.html", &ctx).unwrap());
  }
  let user = user.unwrap();
  Article::add(&mut conn, Article::new(user.id, data.title, data.content)).unwrap();
  HttpResponse::Found().append_header(("Location", "/")).finish()
}

#[get("/article/{id}")]
async fn garticle(pool: Data<DbPool>, token: CsrfToken, user: Option<Identity>, tera: web::Data<Tera>, ath: web::Path<(u32,)>) -> PiggyResult<impl Responder> {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let mut ctx = Context::new();
  if let Some(user) = user {
    if let Ok(user) = User::by_username(&mut conn, &user.id().unwrap()) {
      ctx.insert("user", &user);
      ctx.insert("uacl", &(user.acl() as i32));
    }
  } else { ctx.insert("uacl", &(Acl::All as i32)); }
  let arti = Article::by_id(&mut conn, ath.into_inner().0 as i32);
  if let Err(_) = arti { return Ok(HttpResponse::NotFound().finish()); }
  let arti = arti.unwrap();
  let author = arti.author(&mut conn).unwrap();
  let comment = arti.get_all_comments(&mut conn).unwrap();
  let comment_authors = arti.get_all_comment_authors(&mut conn)?;
  ctx.insert("article", &arti);
  ctx.insert("acl", &arti.acl(&mut conn)?);
  ctx.insert("author", &author);
  ctx.insert("amd5", &md5(&author.email));
  ctx.insert("csrf_token", &token.get());
  ctx.insert("likes", &Like::get_all(&mut conn, arti.id).unwrap());
  ctx.insert("dislikes", &Dislike::get_all(&mut conn, arti.id).unwrap());
  ctx.insert("comments", &comment);
  ctx.insert("comment_authors", &comment_authors);
  let result = tera.render("view.html", &ctx).unwrap();
  Ok(HttpResponse::Ok().body(result))
}

#[post("/article/{id}/delete")]
async fn partidelete(pool: Data<DbPool>, user: Option<Identity>, ath: web::Path<(u32,)>) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let user = User::by_username(&mut conn, &user.unwrap().id().unwrap());
  if let Err(_) = user { return HttpResponse::Unauthorized().finish(); }
  let user = user.unwrap();
  let arti = Article::by_id(&mut conn, ath.into_inner().0 as i32);
  if let Err(_) = arti { return HttpResponse::NotFound().finish(); }
  let arti = arti.unwrap();
  let acl = ArticleAcl::by_article_id(&mut conn, arti.id).unwrap().pdelete as i32;
  if ((acl == (Acl::Author as i32) && user.id != arti.author_id)) && ((user.acl() as i32) < acl) { return HttpResponse::Unauthorized().finish(); }
  arti.delete(&mut conn).unwrap();
  HttpResponse::Found().append_header(("Location", "/")).finish()
}

#[post("/article/{aid}/c/{cid}/delete")]
async fn particomdelete(pool: Data<DbPool>, user: Option<Identity>, ath: web::Path<(u32,u32)>) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let user = User::by_username(&mut conn, &user.unwrap().id().unwrap());
  if let Err(_) = user { return HttpResponse::Unauthorized().finish(); }
  let user = user.unwrap();
  let ath = ath.into_inner();
  let arti = Article::by_id(&mut conn, ath.0 as i32);
  if let Err(_) = arti { return HttpResponse::NotFound().finish(); }
  let arti = arti.unwrap();
  let comment = Comment::by_id(&mut conn, ath.1 as i32);
  if let Err(_) = comment { return HttpResponse::NotFound().finish(); }
  let comment = comment.unwrap();
  let acl = ArticleAcl::by_article_id(&mut conn, arti.id).unwrap().pdelete_comments as i32;
  if ((acl == (Acl::Author as i32) && user.id != comment.author_id)) && ((user.acl() as i32) < acl) { return HttpResponse::Unauthorized().finish(); }
  comment.delete(&mut conn).unwrap();
  HttpResponse::Found().append_header(("Location", format!("/article/{}", arti.id))).finish()
}

#[post("/article/{aid}/c/{cid}/edit")]
async fn particomedit(pool: Data<DbPool>, user: Option<Identity>, data: Form<CommentEditData>, ath: web::Path<(u32,u32)>) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let user = User::by_username(&mut conn, &user.unwrap().id().unwrap());
  if let Err(_) = user { return HttpResponse::Unauthorized().finish(); }
  let user = user.unwrap();
  let ath = ath.into_inner();
  let arti = Article::by_id(&mut conn, ath.0 as i32);
  if let Err(_) = arti { return HttpResponse::NotFound().finish(); }
  let arti = arti.unwrap();
  let comment = Comment::by_id(&mut conn, ath.1 as i32);
  if let Err(_) = comment { return HttpResponse::NotFound().finish(); }
  let comment = comment.unwrap();
  let acl = ArticleAcl::by_article_id(&mut conn, arti.id).unwrap().pedit_comments as i32;
  if ((acl == (Acl::Author as i32) && user.id != comment.author_id)) && ((user.acl() as i32) < acl) { return HttpResponse::Unauthorized().finish(); }
  comment.edit(&mut conn, data.into_inner().content).unwrap();
  HttpResponse::Found().append_header(("Location", format!("/article/{}", arti.id))).finish()
}


#[post("/article/{id}/comment")]
async fn pcomment(pool: Data<DbPool>, user: Option<Identity>, data: Form<CommentWriteData>, ath: web::Path<(u32,)>) -> impl Responder {
  let data = data.into_inner();
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let user = User::by_username(&mut conn, &user.unwrap().id().unwrap());
  if let Err(_) = user { return HttpResponse::Unauthorized().finish(); }
  if data.content.is_empty() { return HttpResponse::Found().append_header(("Location", format!("/article/{}", ath.into_inner().0))).finish(); }
  let user = user.unwrap();
  let arti = Article::by_id(&mut conn, ath.into_inner().0 as i32);
  if let Err(_) = arti { return HttpResponse::NotFound().finish(); }
  let arti = arti.unwrap();
  let acl = arti.acl(&mut conn).unwrap().pwrite_comments as i32;
  if ((acl == (Acl::Author as i32) && user.id != arti.author_id)) && ((user.acl() as i32) < acl) { return HttpResponse::Unauthorized().finish(); }
  Comment::add(&mut conn, Comment::new(arti.id, user.id, data.content)).unwrap();
  HttpResponse::Found().append_header(("Location", format!("/article/{}", arti.id))).finish()
}

#[get("/moonrin")]
async fn moonrin() -> impl Responder { HttpResponse::InternalServerError().finish() }

#[get("/teapot")]
async fn teapot() -> impl Responder { HttpResponse::ImATeapot().finish() }

#[get("/logout")]
async fn logout(id: Option<Identity>) -> impl Responder {
  if let None = id { return HttpResponse::Unauthorized().finish(); }
  id.unwrap().logout();
  HttpResponse::Found().append_header(("Location", "/")).finish()
}

#[get("/article/{id}/change_acl")]
async fn change_acl(pool: Data<DbPool>, user: Option<Identity>, tera: web::Data<Tera>, arti: web::Path<(u32,)>) -> impl Responder {
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let user = User::by_username(&mut conn, &user.unwrap().id().unwrap());
  if let Err(_) = user { return HttpResponse::Unauthorized().finish(); }
  let user = user.unwrap();
  let arti = Article::by_id(&mut conn, arti.into_inner().0 as i32);
  if let Err(_) = arti { return HttpResponse::NotFound().finish(); }
  let arti = arti.unwrap();
  if (user.acl() as i32) < (Acl::Admin as i32) { return HttpResponse::Unauthorized().finish(); }
  let acls = vec!["ban","all","user","email","author","admin","owner"].iter().map(|a|a.to_string()).collect::<Vec<String>>();
  let mut acls = acls.iter().enumerate().collect::<Vec<(usize, &String)>>();
  acls.reverse();
  let mut ctx = Context::new();
  ctx.insert("acls", &acls);
  ctx.insert("ac", &arti.acl(&mut conn).unwrap());
  ctx.insert("article", &arti);
  let result = tera.render("acl.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/article/{id}/change_acl")]
async fn change_acl_post(pool: Data<DbPool>, user: Option<Identity>, data: Form<AclData>, ath: web::Path<(u32,)>) -> impl Responder {
  if let None = user { return HttpResponse::Unauthorized().finish(); }
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let user = User::by_username(&mut conn, &user.unwrap().id().unwrap());
  if let Err(_) = user { return HttpResponse::Unauthorized().finish(); }
  let user = user.unwrap();
  if (user.acl() as i32) < (Acl::Admin as i32) { return HttpResponse::Unauthorized().finish(); }
  let arti = Article::by_id(&mut conn, ath.into_inner().0 as i32);
  if let Err(_) = arti { return HttpResponse::NotFound().finish(); }
  let arti = arti.unwrap();
  let acl = arti.acl(&mut conn).unwrap();
  let data = data.into_inner();
  if let Err(_) = acl.update(&mut conn, data) { return HttpResponse::InternalServerError().finish(); }
  HttpResponse::Found().append_header(("Location", format!("/article/{}", arti.id))).finish()
}

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

#[derive(Deserialize)]
pub struct CommentEditData {
  csrf_token: CsrfToken,
  pub content: String,
}

impl CsrfGuarded for ArticleWriteData { fn csrf_token(&self) -> &CsrfToken { &self.csrf_token } }
impl CsrfGuarded for CommentWriteData { fn csrf_token(&self) -> &CsrfToken { &self.csrf_token } }
impl CsrfGuarded for CommentEditData { fn csrf_token(&self) -> &CsrfToken { &self.csrf_token } }

pub fn services(cfg: &mut web::ServiceConfig) {
  cfg.service(index);
  cfg.service(moonrin);
  cfg.service(garticle);
  cfg.service(gwrite);
  cfg.service(pwrite);
  cfg.service(gupdates);
  cfg.service(pcomment);
  cfg.service(guser);
  cfg.service(teapot);
  cfg.service(logout);
  cfg.service(partidelete);
  cfg.service(particomdelete);
  cfg.service(particomedit);
  cfg.service(change_acl);
  cfg.service(change_acl_post);
}