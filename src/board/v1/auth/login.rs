use actix_csrf::extractor::{CsrfToken, CsrfGuarded};
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{
  web::{self, Form, Data}, get, post, HttpResponse, Responder, HttpRequest, HttpMessage
};
use diesel::{SqliteConnection, r2d2::{self, PooledConnection}};
use piggyboard::{models::User, DbPool};
use serde::Deserialize;
use tera::Tera;

#[get("/signin")]
async fn gsignin() -> impl Responder { HttpResponse::Found().append_header(("Location", "/v3/signin/identifier")).finish() }

#[get("/login")]
async fn glogin() -> impl Responder { HttpResponse::Found().append_header(("Location", "/v3/signin/identifier")).finish() }

#[get("/v3/signin/identifier")]
async fn gsignin_v3_identifier(token: CsrfToken, tera: web::Data<Tera>) -> impl Responder {
  let mut ctx = tera::Context::new();
  ctx.insert("csrf_token", &token.get());
  let result = tera.render("identifier.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/v3/signin/identifier")]
async fn psignin_v3_identifier(
  tera: Data<Tera>,
  pool: Data<DbPool>,
  form: Form<SigninV3Identifier>,
  session: Session,
) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let form = form.into_inner();
  let user = User::by_username(&mut conn, &form.login);
  if user.is_ok() {
    session.insert("login", form.login).unwrap();
    return HttpResponse::Found().append_header(("Location", "/v3/signin/challenge/pwd")).finish();
  }
  let mut ctx = tera::Context::new();
  ctx.insert("ierr", if !form.login.is_empty() {"Fishydino 계정을 찾을 수 없습니다."}else{"아이디를 입력해주세요."});
  ctx.insert("csrf_token", &form.csrf_token.get());
  let result = tera.render("identifier.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[get("/v3/signin/challenge/pwd")]
async fn gsignin_v3_challenge_pwd(token: CsrfToken, tera: Data<Tera>, session: Session) -> impl Responder {
  if session.get::<String>("login").unwrap().is_none() { return HttpResponse::Found().append_header(("Location", "/v3/signin/identifier")).finish(); }
  let mut ctx = tera::Context::new();
  ctx.insert("csrf_token", &token.get());
  ctx.insert("login", &session.get::<String>("login").unwrap().unwrap());
  let result = tera.render("challenge_pwd.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/v3/signin/challenge/pwd")]
async fn psignin_v3_challenge_pwd(request: HttpRequest, pool: Data<DbPool>, session: Session, tera: web::Data<Tera>, pw: Form<SigninV3ChallengePwd>) -> impl Responder {
  if session.get::<String>("login").unwrap().is_none() { return HttpResponse::Found().append_header(("Location", "/v3/signin/identifier")).finish(); }
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let user = User::by_username(&mut conn, &session.get::<String>("login").unwrap().unwrap());
  if user.is_err() { return HttpResponse::Found().append_header(("Location", "/v3/signin/identifier")).finish(); }
  let user = user.unwrap();
  let pw = pw.into_inner();
  let token = pw.csrf_token;
  let pw = pw.lpass;
  let correct = match User::password_check(&user, &pw) {
    Ok(correct) => correct,
    Err(_) => return HttpResponse::Found().append_header(("Location", "/info/unknownerror")).finish(),
  };
  if correct {
    Identity::login(&request.extensions(), user.username.into()).unwrap();
    return HttpResponse::Found().append_header(("Location", "/")).finish();
  }
  let mut ctx = tera::Context::new();
  ctx.insert("login", &session.get::<String>("login").unwrap().unwrap());
  ctx.insert("perr", "잘못된 비밀번호입니다. 다시 시도하거나 비밀번호 찾기를 클릭하여 재설정하세요.");
  ctx.insert("csrf_token", &token.get());
  let result = tera.render("challenge_pwd.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

pub fn services(cfg: &mut web::ServiceConfig) {
  cfg.service(gsignin);
  cfg.service(glogin);
  cfg.service(gsignin_v3_identifier);
  cfg.service(psignin_v3_identifier);
  cfg.service(gsignin_v3_challenge_pwd);
  cfg.service(psignin_v3_challenge_pwd);
}

#[derive(Deserialize)]
pub struct SigninV3Identifier {
  csrf_token: CsrfToken,
  login: String,
}

#[derive(Deserialize)]
pub struct SigninV3ChallengePwd {
  lpass: String,
  csrf_token: CsrfToken,
}

impl CsrfGuarded for SigninV3Identifier { fn csrf_token(&self) -> &CsrfToken { &self.csrf_token } }
impl CsrfGuarded for SigninV3ChallengePwd { fn csrf_token(&self) -> &CsrfToken { &self.csrf_token } }