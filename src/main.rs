use actix_web::{
  http::{header, StatusCode}, cookie::Key, App, HttpServer,
  dev::ServiceResponse, middleware::{Logger, ErrorHandlerResponse, ErrorHandlers},
  web::{self, Data}, Result as ActixResult, HttpRequest, HttpResponse
};
use actix_identity::IdentityMiddleware;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_files::Files;
use diesel::{SqliteConnection, r2d2};
use env_logger::Env;
use piggyboard::error::{PiggyError, PiggyErrorKind};
use tera::Tera;
use std::io::Result as IOResult;
use dotenv::dotenv;
use r2d2::{ConnectionManager, Pool};

mod board;

fn handle_notfound<B>(mut res: ServiceResponse<B>) -> ActixResult<ErrorHandlerResponse<B>> {
  res.response_mut().headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/html; charset=utf-8"));
  let (req, res): (HttpRequest, HttpResponse<B>) = res.into_parts();
  let res: actix_web::HttpResponse<String> = res.set_body(std::fs::read_to_string("./public/views/404.html").unwrap());
  let res: ServiceResponse<actix_web::body::EitherBody<B>> = ServiceResponse::new(req, res).map_into_boxed_body().map_into_right_body();
  Ok(ErrorHandlerResponse::Response(res))
}

#[actix_web::main]
async fn main() -> IOResult<()> {
  dotenv().ok();
  env_logger::init_from_env(Env::default().default_filter_or("info"));
  let tera = Tera::new("public/views/**/*.html").unwrap();
  let secret_key: Key = Key::generate();
  let manager: ConnectionManager::<SqliteConnection>  = ConnectionManager::<SqliteConnection>::new("./piggyboard.sqlite3");
  let pool: Pool<ConnectionManager<SqliteConnection>> = Pool::builder()
    .build(manager)
    .expect("database URL should be valid path to SQLite DB file");
  HttpServer::new(move || {
    // let cors: Cors = Cors::default()
    //   .allowed_methods(vec!["GET", "POST"])
    //   .allowed_headers(vec![
    //     http::header::AUTHORIZATION,
    //     http::header::ACCEPT,
    //   ])
    //   .allowed_header(http::header::CONTENT_TYPE);
    App::new()
      .wrap(Logger::default())
      .wrap(Logger::new("%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T"))
      .wrap(ErrorHandlers::new().handler(StatusCode::NOT_FOUND, handle_notfound))
      .wrap(IdentityMiddleware::default())
      .wrap(SessionMiddleware::new(CookieSessionStore::default(), secret_key.clone()))
      .app_data(Data::new(tera.clone()))
      .app_data(Data::new(pool.clone()))
      .app_data(web::PathConfig::default().error_handler(|_err, _req| {
        PiggyError::from_kind(PiggyErrorKind::MimeUnknown).into()
      }))
      .configure(board::v1::services)
      .service(Files::new("/", "static/"))
      // .service(web::scope("/v1").configure(api::v1::services))
  })
  .bind(("127.0.0.1", 23519))?
  .run()
  .await
}