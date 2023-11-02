use actix_session::Session;
use actix_web::{
  web::{self, Form, Data}, get, post, HttpResponse, Responder
};
use chrono::NaiveDate;
use diesel::{SqliteConnection, r2d2::{self, PooledConnection}};
use piggyboard::{models::User, DbPool};
use serde::Deserialize;
use tera::Tera;

#[get("/info/unknownerror")]
async fn gunknownerror(tera: web::Data<Tera>) -> impl Responder { HttpResponse::Ok().body(ret_error_page(&tera, &"문제가 발생했습니다. 다시 시도해 주세요.")) }

#[get("/signup")]
async fn gsignup() -> impl Responder { HttpResponse::Found().append_header(("Location", "/signup/v2/createaccount")).finish() }

#[get("/signup/v2/createaccount")]
async fn gsignup_v2_createaccount(tera: web::Data<Tera>) -> impl Responder {
  let result = tera.render("createaccount.html", &tera::Context::new()).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/signup/v2/createaccount")]
async fn psignup_v2_createaccount(session: Session, tera: web::Data<Tera>, cadata: Form<CreateAccount>) -> impl Responder {
  let account = cadata.into_inner();
  let mut ctx = tera::Context::new();
  if account.firstname == "" {
    ctx.insert("ferr", "이름을 정확하게 입력하셨나요?");
  } else {
    session.insert("firstname", account.firstname).unwrap();
    session.insert("lastname", account.lastname.unwrap_or("".to_string())).unwrap();
    return HttpResponse::Found().append_header(("Location", "/signup/v2/birthdaygender")).finish();
  }
  let result = tera.render("createaccount.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[get("/signup/v2/birthdaygender")]
async fn gsignup_v2_birthdaygender(session: Session, tera: web::Data<Tera>) -> impl Responder {
  if session.get::<String>("firstname").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/createaccount")).finish(); }
  let ctx = tera::Context::new();
  let result = tera.render("birthdaygender.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/signup/v2/birthdaygender")]
async fn psignup_v2_birthdaygender(session: Session, tera: web::Data<Tera>, bgdata: Form<BirthdayGender>) -> impl Responder {
  if session.get::<String>("firstname").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/createaccount")).finish(); }
  let birgen = bgdata.into_inner();
  let birth = parse_date(&birgen.birth);
  let mut ctx = tera::Context::new();
  if birth.is_err() {
    ctx.insert("berr", "생년월일을 정확하게 입력하셨나요?");
  } else if birth.unwrap() < NaiveDate::from_ymd_opt(1923, 1, 1).unwrap() {
    ctx.insert("berr", "어떻게 살아있냐 넌ㅋㅋㅋㅋ");
  } else {
    session.insert("birth", birth.unwrap()).unwrap();
    session.insert("gender", birgen.gender).unwrap();
    return HttpResponse::Found().append_header(("Location", "/signup/v2/createusername")).finish();
  }
  let result = tera.render("birthdaygender.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[get("/signup/v2/createusername")]
async fn gsignup_v2_createusername(session: Session, tera: web::Data<Tera>) -> impl Responder {
  if session.get::<String>("firstname").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/createaccount")).finish(); }
  if session.get::<NaiveDate>("birth").unwrap() == None || session.get::<String>("gender").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/birthdaygender")).finish(); }
  let ctx = tera::Context::new();
  let result = tera.render("createusername.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/signup/v2/createusername")]
async fn psignup_v2_createusername(pool: Data<DbPool>, session: Session, tera: web::Data<Tera>, bgdata: Form<CreateUsername>) -> impl Responder {
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  if session.get::<String>("firstname").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/createaccount")).finish(); }
  if session.get::<NaiveDate>("birth").unwrap() == None || session.get::<String>("gender").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/birthdaygender")).finish(); }
  let username = bgdata.into_inner().userid;
  let mut ctx = tera::Context::new();
  let disallowed: Vec<String> = vec!["admin", "administrator", "moderator", "mod", "piggyboard", "piggy", "board", "administ", "administra", "admin"].iter().map(|s| s.to_string()).collect();
  match User::by_username(&mut conn, &username) {
    Ok(_) => {ctx.insert("uerr", "이미 사용된 사용자 이름입니다. 다른 이름을 선택하세요.");},
    Err(_) => {
      if !disallowed.contains(&username) {
        session.insert("username", username).unwrap();
        return HttpResponse::Found().append_header(("Location", "/signup/v2/createpassword")).finish();
      }
      ctx.insert("uerr", "허용되지 않는 사용자 이름입니다. 다시 시도해 주세요.");
    }
  }
  let result = tera.render("createusername.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[get("/signup/v2/createpassword")]
async fn gsignup_v2_createpassword(session: Session, tera: web::Data<Tera>) -> impl Responder {
  if session.get::<String>("firstname").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/createaccount")).finish(); }
  if session.get::<NaiveDate>("birth").unwrap() == None || session.get::<String>("gender").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/birthdaygender")).finish(); }
  if session.get::<String>("username").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/createusername")).finish(); }
  let ctx = tera::Context::new();
  let result = tera.render("createpassword.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[post("/signup/v2/createpassword")]
async fn psignup_v2_createpassword(session: Session, tera: web::Data<Tera>, pwdata: Form<CreatePassword>) -> impl Responder {
  if session.get::<String>("firstname").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/createaccount")).finish(); }
  if session.get::<NaiveDate>("birth").unwrap() == None || session.get::<String>("gender").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/birthdaygender")).finish(); }
  if session.get::<String>("username").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/createusername")).finish(); }
  let pw = pwdata.into_inner();
  let mut ctx = tera::Context::new();
  if pw.password == pw.pwcheck {
    session.insert("password", pw.password).unwrap();
    return HttpResponse::Found().append_header(("Location", "/signup/v2/complete")).finish();
  } else {
    ctx.insert("cerr", "비밀번호가 일치하지 않았습니다. 다시 시도해 보세요.");
  }
  let result = tera.render("createpassword.html", &ctx).unwrap();
  HttpResponse::Ok().body(result)
}

#[get("/signup/v2/complete")]
async fn gsignup_v2_complete(pool: Data<DbPool>, session: Session) -> impl Responder {
  if session.get::<String>("firstname").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/createaccount")).finish(); }
  if session.get::<NaiveDate>("birth").unwrap() == None || session.get::<String>("gender").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/birthdaygender")).finish(); }
  if session.get::<String>("username").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/createusername")).finish(); }
  if session.get::<String>("password").unwrap() == None { return HttpResponse::Found().append_header(("Location", "/signup/v2/createpassword")).finish(); }
  let nuser = match User::new(
    session.get::<String>("username").unwrap().unwrap(),
    session.get::<String>("firstname").unwrap().unwrap(),
    session.get::<String>("lastname").unwrap(),
    session.get::<String>("gender").unwrap(),
    format!("{}@urdekcah.me", session.get::<String>("username").unwrap().unwrap()),
    session.get::<String>("password").unwrap().unwrap(),
  ) {
    Ok(u) => u,
    Err(_) => return HttpResponse::Found().append_header(("Location", "/info/unknownerror")).finish()
  };
  let mut conn: PooledConnection<r2d2::ConnectionManager<SqliteConnection>> = pool.get().unwrap();
  let _ = match User::add(&mut conn, nuser) {
    Ok(u) => u,
    Err(_) => return HttpResponse::Found().append_header(("Location", "/info/unknownerror")).finish()
  };
  return HttpResponse::Found().append_header(("Location", "/")).finish()
}

#[derive(Deserialize)]
pub struct CreateAccount {
  pub lastname: Option<String>,
  pub firstname: String,
}

#[derive(Deserialize)]
pub struct BirthdayGender {
  pub birth: String,
  pub gender: String,
}

#[derive(Deserialize)]
pub struct CreateUsername {
  pub userid: String,
}

#[derive(Deserialize)]
pub struct CreatePassword {
  pub password: String,
  pub pwcheck: String
}

pub fn services(cfg: &mut web::ServiceConfig) {
  cfg.service(gunknownerror);
  cfg.service(gsignup);
  cfg.service(gsignup_v2_createaccount);
  cfg.service(psignup_v2_createaccount);
  cfg.service(gsignup_v2_birthdaygender);
  cfg.service(psignup_v2_birthdaygender);
  cfg.service(gsignup_v2_createusername);
  cfg.service(psignup_v2_createusername);
  cfg.service(gsignup_v2_createpassword);
  cfg.service(psignup_v2_createpassword);
  cfg.service(gsignup_v2_complete);
}

fn parse_date(date_str: &str) -> Result<NaiveDate, chrono::ParseError> {
  let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;
  Ok(date)
}

fn ret_error_page(tera: &Tera, err: &impl ToString) -> String {
  let mut ctx = tera::Context::new();
  ctx.insert("err", &err.to_string());
  tera.render("error.html", &ctx).unwrap()
}

/*
전화번호
우리는 이 번호를 계정 보안 용도로만 사용합니다. 다른 사용자에게는 전화번호가 표시되지 않습니다. 나중에 다른 용도로도 이 번호를 사용할지 결정할 수 있습니다.
 */
