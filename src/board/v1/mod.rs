use actix_web::web::ServiceConfig;

pub mod meta;
pub mod auth;

pub fn services(cfg: &mut ServiceConfig) {
  meta::services(cfg);
  auth::services(cfg);
}