pub mod error;
pub mod res;
pub mod models;
pub mod cryp;
pub mod schema;

use error::{PiggyError, PiggyErrorKind};
use models::User;
use serde::{Serialize, Deserialize};
use diesel::{prelude::*, sqlite::SqliteConnection, r2d2};

pub type StdResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type DieselResult<T> = Result<T, diesel::result::Error>;
pub type PiggyResult<T> = Result<T, error::PiggyError>;
pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

#[cfg(debug_assertions)]
pub static ROOT_HOST: &'static str = "127.0.0.1";
#[cfg(not(debug_assertions))]
pub static ROOT_HOST: &'static str = "sunrin2023.urdekcah.me";

pub fn establish_connection() -> SqliteConnection {
  let database_url: String = String::from("sqlite:///C:/rust/piggyboard/piggyboard.sqlite3");
  SqliteConnection::establish(&database_url)
    .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserData {
  pub profile_img: Option<String>,
  pub profile_thumbnail: Option<String>,
  pub profile_banner: Option<String>,
  pub first_name: String,
  pub last_name: Option<String>,
  pub gender: Option<String>,
  pub username: String,
  pub email: String,
  pub phone: Option<String>,
  pub lang: Option<String>,
  pub bio: Option<String>,
  pub password: String
}

#[derive(Serialize, Deserialize)]
pub struct UserQuery {
  pub userid: Option<i32>,
  pub username: Option<String>,
  pub email: Option<String>,
}

impl UserQuery {
  pub fn get(conn: &mut SqliteConnection, data: &UserQuery) ->PiggyResult<User> {
    match data {
      UserQuery { userid: Some(id), .. } => User::by_id(conn, *id),
      UserQuery { username: Some(username), .. } => User::by_username(conn, &username),
      UserQuery { email: Some(email), .. } => User::by_email(conn, &email),
      _ => { Err(PiggyError::from_kind(PiggyErrorKind::UserQueryRequired)) }
    }
  }
}