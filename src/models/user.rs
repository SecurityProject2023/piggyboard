use diesel::prelude::*;
use crate::{schema::users, CreateUserData, error::{PiggyError, PiggyErrorKind}, PiggyResult, cryp::{password_hash, verify_password}};
use chrono::NaiveDateTime;
use serde::Serialize;
use regex::Regex;

const EMAIL_REGEX: &'static str = r"(?x)^(?P<login>[^@\s]+)@([[:word:]]+\.)+[[:word:]]+$";
const USERNAME_REGEX: &'static str = r"(?x)^[a-zA-Z0-9_]{1,15}$";
const PHONE_REGEX: &'static str = r"^\+[1-9]{1}[0-9]{3,14}$";

const DEFAULT_PROFILE_IMG: &'static str = "https://static.fishydi.no/img/defaultProfileImg.png";
const DEFAULT_PROFILE_THUMBNAIL: &'static str = "https://static.fishydi.no/img/defaultProfileThumbnail.png";
const DEFAULT_PROFILE_BANNER: &'static str = "https://static.fishydi.no/img/defaultProfileBanner.png";
const DEFAULT_BIO: &'static str = "PiggyBoard는 처음 써봐요. 환영해 주세요!";

#[derive(Queryable, Debug, PartialEq, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
  pub id: i32,
  pub profile_img: String,
  pub profile_thumbnail: String,
  pub profile_banner: String,
  pub first_name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub last_name: Option<String>,
  pub gender: String,
  pub username: String,
  pub email: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub phone: Option<String>,
  pub lang: String,
  pub bio: String,
  #[serde(skip)]
  pub password: String,
  pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
  pub profile_img: String,
  pub profile_thumbnail: String,
  pub profile_banner: String,
  pub first_name: String,
  pub last_name: Option<String>,
  pub gender: String,
  pub username: String,
  pub email: String,
  pub phone: Option<String>,
  pub lang: String,
  pub bio: String,
  pub password: String,
  pub created_at: NaiveDateTime,
}

impl User {
  pub fn new<U, F, L, G, E, P>(uname: U, fname: F, lname: Option<L>, gender: Option<G>, mail: E, upw: P) -> PiggyResult<NewUser>
    where U: ToString, F: ToString, L: ToString, G: ToString, E: ToString, P: ToString {
      let password = password_hash(upw.to_string().as_str())?;
      Ok(NewUser {
        profile_img: String::from(DEFAULT_PROFILE_IMG),
        profile_thumbnail: String::from(DEFAULT_PROFILE_THUMBNAIL),
        profile_banner: String::from(DEFAULT_PROFILE_BANNER),
        first_name: fname.to_string(),
        last_name: lname.map_or(None, |v: L| Some(v.to_string())),
        gender: gender.map_or(String::from("unknown"), |v: G| v.to_string()),
        username: uname.to_string(),
        email: mail.to_string(),
        phone: None,
        lang: String::from("ko"),
        bio: String::from(DEFAULT_BIO),
        password,
        created_at: chrono::Utc::now().naive_utc()
      })
  }

  pub fn from_req(pload: CreateUserData) -> PiggyResult<NewUser> {
    if !User::email_vaild(&pload.email) { return Err(PiggyError::from_kind(PiggyErrorKind::InvalidEmail)); }
    if !User::username_vaild(&pload.username) { return Err(PiggyError::from_kind(PiggyErrorKind::InvalidUsername)); }
    if let Some(phone) = &pload.phone { 
      if !User::phone_vaild(phone) { return Err(PiggyError::from_kind(PiggyErrorKind::InvalidPhone)); }
    }
    let password = password_hash(&pload.password)?;
    Ok(NewUser {
      profile_img: pload.profile_img.map_or(DEFAULT_PROFILE_IMG.to_string(), |v| v),
      profile_thumbnail: pload.profile_thumbnail.map_or(DEFAULT_PROFILE_THUMBNAIL.to_string(), |v| v),
      profile_banner: pload.profile_banner.map_or(DEFAULT_PROFILE_BANNER.to_string(), |v| v),
      first_name: pload.first_name,
      last_name: pload.last_name.map_or(None, |v: String| Some(v)),
      gender: pload.gender.map_or("unknown".to_string(), |v: String| v),
      username: pload.username,
      email: pload.email,
      phone: pload.phone.map_or(None, |v: String| Some(v)),
      lang: pload.lang.map_or("ko".to_string(), |v: String| v),
      bio: pload.bio.map_or(DEFAULT_BIO.to_string(), |v: String| v),
      password,
      created_at: chrono::Utc::now().naive_utc()
    })
  }

  pub fn email_in_use(conn: &mut SqliteConnection, email: impl ToString) -> bool {
    match users::table.filter(users::email.eq(email.to_string())).load::<User>(conn) {
      Ok(users) => users.len() > 0,
      Err(_) => true
    }
  }

  pub fn username_in_use(conn: &mut SqliteConnection, username: &impl ToString) -> bool {
    match users::table.filter(users::username.eq(username.to_string())).load::<User>(conn) {
      Ok(users) => users.len() > 0,
      Err(_) => true
    }
  }

  pub fn phone_in_use(conn: &mut SqliteConnection, phone: impl ToString) -> bool {
    match users::table.filter(users::phone.eq(phone.to_string())).load::<User>(conn) {
      Ok(users) => users.len() > 0,
      Err(_) => true
    }
  }

  pub fn by_id(conn: &mut SqliteConnection, user_id: i32) -> PiggyResult<User> {
    match users::table.filter(users::id.eq(user_id)).first::<User>(conn) {
      Ok(user) => Ok(user),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::UserNotFound))
    }
  }

  pub fn by_username(conn: &mut SqliteConnection, username: &impl ToString) -> PiggyResult<User> {
    match users::table.filter(users::username.eq(username.to_string())).first::<User>(conn) {
      Ok(user) => Ok(user),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::UserNotFound))
    }
  }

  pub fn by_email(conn: &mut SqliteConnection, email: &impl ToString) -> PiggyResult<User> {
    if !User::email_vaild(email) { return Err(PiggyError::from_kind(PiggyErrorKind::InvalidEmail)); }
    match users::table.filter(users::email.eq(email.to_string())).first::<User>(conn) {
      Ok(user) => Ok(user),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::UserNotFound))
    }
  }

  pub fn password_check(user: &User, password: &impl ToString) -> PiggyResult<bool> {
    verify_password(&user.password, &password.to_string())
  }

  pub fn change_username(conn: &mut SqliteConnection, user: &User, change: &impl ToString) -> PiggyResult<User> {
    if !User::username_vaild(change) { return Err(PiggyError::from_kind(PiggyErrorKind::InvalidUsername)); }
    if User::username_in_use(conn, change) { return Err(PiggyError::from_kind(PiggyErrorKind::UsernameAlreadyInUse)); }
    diesel::update(users::table.filter(users::id.eq(user.id)))
      .set(users::username.eq(change.to_string()))
      .execute(conn)?;
    Ok(users::table.filter(users::id.eq(user.id)).first(conn)?)
  }

  pub fn change_password(conn: &mut SqliteConnection, user: &User, current: &impl ToString, change: &impl ToString) -> PiggyResult<User> {
    if current.to_string() == change.to_string() { return Err(PiggyError::from_kind(PiggyErrorKind::PasswordIsSame)); }
    if !User::password_check(user, current)? { return Err(PiggyError::from_kind(PiggyErrorKind::PasswordIncorrect)); }
    diesel::update(users::table.filter(users::id.eq(user.id)))
      .set(users::password.eq(password_hash(&change.to_string())?))
      .execute(conn)?;
    Ok(users::table.filter(users::id.eq(user.id)).first(conn)?)
  }

  pub fn change_name(conn: &mut SqliteConnection, user: &User, firstname: &impl ToString, lastname: &Option<impl ToString>) -> PiggyResult<User> {
    let lastname: String = match lastname { Some(v) => v.to_string(), None => "".to_string() };
    diesel::update(users::table.filter(users::id.eq(user.id)))
      .set((users::first_name.eq(firstname.to_string()), users::last_name.eq(lastname)))
      .execute(conn)?;
    Ok(users::table.filter(users::id.eq(user.id)).first(conn)?)
  }

  pub fn change_bio(conn: &mut SqliteConnection, user: &User, bio: &impl ToString) -> PiggyResult<User> {
    diesel::update(users::table.filter(users::id.eq(user.id)))
      .set(users::bio.eq(bio.to_string()))
      .execute(conn)?;
    Ok(users::table.filter(users::id.eq(user.id)).first(conn)?)
  }

  pub fn change_profile_imgs(
    conn: &mut SqliteConnection,
    user: &User,
    profile_img: &Option<impl ToString>,
    profile_thumbnail: &Option<impl ToString>,
    profile_banner: &Option<impl ToString>
  ) -> PiggyResult<User> {
    let profile_img: String = match profile_img { Some(v) => v.to_string(), None => DEFAULT_PROFILE_IMG.to_string() };
    let profile_thumbnail: String = match profile_thumbnail { Some(v) => v.to_string(), None => DEFAULT_PROFILE_THUMBNAIL.to_string() };
    let profile_banner: String = match profile_banner { Some(v) => v.to_string(), None => DEFAULT_PROFILE_BANNER.to_string() };
    diesel::update(users::table.filter(users::id.eq(user.id)))
      .set((
        users::profile_img.eq(profile_img),
        users::profile_thumbnail.eq(profile_thumbnail),
        users::profile_banner.eq(profile_banner)
      ))
      .execute(conn)?;
    Ok(users::table.filter(users::id.eq(user.id)).first(conn)?)
  }

  pub fn add(conn: &mut SqliteConnection, user: NewUser) -> PiggyResult<User> {
    if User::email_in_use(conn, &user.email) { return Err(PiggyError::from_kind(PiggyErrorKind::EmailAlreadyInUse)); }
    if User::username_in_use(conn, &user.username) { return Err(PiggyError::from_kind(PiggyErrorKind::UsernameAlreadyInUse)); }
    if let Some(phone) = &user.phone {
      if User::phone_in_use(conn, phone) { return Err(PiggyError::from_kind(PiggyErrorKind::PhoneAlreadyInUse)); }
    }
    diesel::insert_into(users::table)
      .values(&user)
      .execute(conn)?;
    Ok(users::table.order(users::id.desc()).first(conn)?)
  }

  // pub fn block(&self, conn: &mut SqliteConnection, target: &User) -> PiggyResult<Block> { Block::add(conn, self, target) }

  // pub fn has_recent_report(&self, conn: &mut SqliteConnection) -> PiggyResult<bool> {
  //   let now: NaiveDateTime = Utc::now().naive_utc();
  //   let one_minute_ago: NaiveDateTime = now - Duration::minutes(1);
  //   let count = reports::table
  //     .filter(reports::reporter_id.eq(self.id))
  //     .filter(reports::created_at.between(one_minute_ago, now))
  //     .count()
  //     .get_result::<i64>(conn)?;
  //   Ok(count > 0)
  // }

  pub fn email_vaild(email: &impl ToString) -> bool { Regex::new(EMAIL_REGEX).unwrap().is_match(&email.to_string()) }
  pub fn username_vaild(username: &impl ToString) -> bool { Regex::new(USERNAME_REGEX).unwrap().is_match(&username.to_string()) }
  pub fn phone_vaild(phone: &impl ToString) -> bool { Regex::new(PHONE_REGEX).unwrap().is_match(&phone.to_string()) }
}