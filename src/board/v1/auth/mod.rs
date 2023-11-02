use actix_web::web::ServiceConfig;

pub mod signup;
pub mod login;

pub fn services(cfg: &mut ServiceConfig) {
  signup::services(cfg);
  login::services(cfg);
}