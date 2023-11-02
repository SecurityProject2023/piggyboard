use actix_web::{web, get, post, HttpResponse, Responder};
use serde_json::json;
use tera::{Tera, Context};

#[get("/")]
async fn index(tera: web::Data<Tera>) -> impl Responder {
  let ctx = Context::new();
  let result = tera.render("index.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/version")]
async fn version() -> impl Responder {
  HttpResponse::Ok().json(json!({
    "version": "0.1.0",
    "name": "Funny Fishy",
  }))
}

pub fn services(cfg: &mut web::ServiceConfig) {
  cfg.service(index);
  cfg.service(version);
}