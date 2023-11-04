use actix_web::{
  http::{header, StatusCode, Method}, cookie::Key, App, HttpServer,
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
use actix_csrf::CsrfMiddleware;
use rand::rngs::StdRng;

mod board;

fn handle_error<B>(mut res: ServiceResponse<B>, error_file: &str) -> ActixResult<ErrorHandlerResponse<B>> {
  res.response_mut().headers_mut().insert(header::CONTENT_TYPE, header::HeaderValue::from_static("text/html; charset=utf-8"));
  let (req, res): (HttpRequest, HttpResponse<B>) = res.into_parts();
  let res: actix_web::HttpResponse<String> = res.set_body(std::fs::read_to_string(error_file).unwrap());
  let res: ServiceResponse<actix_web::body::EitherBody<B>> = ServiceResponse::new(req, res).map_into_boxed_body().map_into_right_body();
  Ok(ErrorHandlerResponse::Response(res))
}

fn handle_unauthorized<B>(res: ServiceResponse<B>) -> ActixResult<ErrorHandlerResponse<B>> { handle_error(res, "./public/views/401.html") }
fn handle_forbidden<B>(res: ServiceResponse<B>) -> ActixResult<ErrorHandlerResponse<B>> { handle_error(res, "./public/views/403.html") }
fn handle_notfound<B>(res: ServiceResponse<B>) -> ActixResult<ErrorHandlerResponse<B>> { handle_error(res, "./public/views/404.html") }
fn handle_internal_server_error<B>(res: ServiceResponse<B>) -> ActixResult<ErrorHandlerResponse<B>> { handle_error(res, "./public/views/500.html") }

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
    let csrf = CsrfMiddleware::<StdRng>::new()
      .set_cookie(Method::GET, "/login")
      .set_cookie(Method::GET, "/article/{id}")
      .set_cookie(Method::GET, "/signup/v2/createaccount")
      .set_cookie(Method::GET, "/signup/v2/birthdaygender")
      .set_cookie(Method::GET, "/signup/v2/createusername")
      .set_cookie(Method::GET, "/signup/v2/createpassword")
      .set_cookie(Method::GET, "/v3/signin/identifier")
      .set_cookie(Method::GET, "/v3/signin/challenge/pwd");
    App::new()
      .wrap(csrf)
      .wrap(Logger::default())
      .wrap(Logger::new("%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T"))
      .wrap(ErrorHandlers::new()
        .handler(StatusCode::UNAUTHORIZED, handle_unauthorized)
        .handler(StatusCode::FORBIDDEN, handle_forbidden)
        .handler(StatusCode::NOT_FOUND, handle_notfound)
        .handler(StatusCode::INTERNAL_SERVER_ERROR, handle_internal_server_error)
      )
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