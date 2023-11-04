use actix_web::web::ServiceConfig;

pub mod rate;

pub fn services(cfg: &mut ServiceConfig) {
  rate::services(cfg);
}