use actix_identity::Identity;
use actix_web::{web::{Data, self}, HttpResponse, post, Responder};
use diesel::{r2d2::{PooledConnection, self}, SqliteConnection};
use piggyboard::{DbPool, models::{User, Article, Like, Dislike}};
use serde_json::json;
use serde::Deserialize;

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

pub fn services(cfg: &mut web::ServiceConfig) {
  cfg.service(prate_article);
}

#[derive(Deserialize)]
pub struct ArticleRateData {
  pub value: i32,
  pub user_id: i32,
  pub csrf_token: String
}