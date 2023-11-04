use diesel::prelude::*;
use crate::{schema::{like, dislike}, PiggyResult, error::{PiggyError, PiggyErrorKind}};
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;

#[derive(Queryable, Debug, PartialEq, Serialize)]
#[diesel(table_name = like)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Like {
  pub id: i32,
  pub user_id: i32,
  pub article_id: i32,
  pub created_at: NaiveDateTime,
}

#[derive(Queryable, Debug, PartialEq, Serialize)]
#[diesel(table_name = dislike)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Dislike {
  pub id: i32,
  pub user_id: i32,
  pub article_id: i32,
  pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = like)]
pub struct NewLike {
  pub user_id: i32,
  pub article_id: i32,
  pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = dislike)]
pub struct NewDislike {
  pub user_id: i32,
  pub article_id: i32,
  pub created_at: NaiveDateTime,
}

impl Like {
  pub fn new(user_id: i32, article_id: i32) -> NewLike {
    NewLike {
      user_id,
      article_id,
      created_at: Utc::now().naive_utc(),
    }
  }

  pub fn liked(conn: &mut SqliteConnection, user_id: i32, article_id: i32) -> PiggyResult<bool> {
    match like::table.filter(like::user_id.eq(user_id)).filter(like::article_id.eq(article_id)).first::<Like>(conn) {
      Ok(_) => Ok(true),
      Err(_) => Ok(false),
    }
  }

  pub fn add(conn: &mut SqliteConnection, post: NewLike) -> PiggyResult<Like> {
    diesel::insert_into(like::table)
      .values(&post)
      .execute(conn)?;
    Ok(like::table.order(like::id.desc()).first(conn)?)
  }

  pub fn by_id(conn: &mut SqliteConnection, id: i32) -> PiggyResult<Like> {
    match like::table.find(id).first(conn) {
      Ok(post) => Ok(post),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::PostNotFound)),
    }
  }

  pub fn get_all(conn: &mut SqliteConnection, article_id: i32) -> PiggyResult<Vec<Like>> {
    Ok(like::table.filter(like::article_id.eq(article_id)).load::<Like>(conn)?)
  }
}

impl Dislike {
  pub fn new(user_id: i32, article_id: i32) -> NewDislike {
    NewDislike {
      user_id,
      article_id,
      created_at: Utc::now().naive_utc(),
    }
  }

  pub fn disliked(conn: &mut SqliteConnection, user_id: i32, article_id: i32) -> PiggyResult<bool> {
    match dislike::table.filter(dislike::user_id.eq(user_id)).filter(dislike::article_id.eq(article_id)).first::<Dislike>(conn) {
      Ok(_) => Ok(true),
      Err(_) => Ok(false),
    }
  }

  pub fn add(conn: &mut SqliteConnection, post: NewDislike) -> PiggyResult<Dislike> {
    diesel::insert_into(dislike::table)
      .values(&post)
      .execute(conn)?;
    Ok(dislike::table.order(dislike::id.desc()).first(conn)?)
  }

  pub fn by_id(conn: &mut SqliteConnection, id: i32) -> PiggyResult<Dislike> {
    match dislike::table.find(id).first(conn) {
      Ok(post) => Ok(post),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::PostNotFound)),
    }
  }

  pub fn get_all(conn: &mut SqliteConnection, article_id: i32) -> PiggyResult<Vec<Dislike>> {
    Ok(dislike::table.filter(dislike::article_id.eq(article_id)).load::<Dislike>(conn)?)
  }
}