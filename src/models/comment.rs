use std::collections::HashMap;

use diesel::prelude::*;
use crate::{schema::comments, PiggyResult, error::{PiggyError, PiggyErrorKind}};
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;

use super::User;

#[derive(Queryable, Debug, PartialEq, Serialize)]
#[diesel(table_name = comments)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Comment {
  pub id: i32,
  pub article_id: i32,
  pub author_id: i32,
  pub content: String,
  pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = comments)]
pub struct NewComment {
  pub article_id: i32,
  pub author_id: i32,
  pub content: String,
  pub created_at: NaiveDateTime,
}

impl Comment {
  pub fn new(article_id: i32, author_id: i32, content: String) -> NewComment {
    NewComment {
      article_id,
      author_id,
      content,
      created_at: Utc::now().naive_utc(),
    }
  }

  pub fn add(conn: &mut SqliteConnection, post: NewComment) -> PiggyResult<Comment> {
    diesel::insert_into(comments::table)
      .values(&post)
      .execute(conn)?;
    Ok(comments::table.order(comments::id.desc()).first(conn)?)
  }

  pub fn by_id(conn: &mut SqliteConnection, id: i32) -> PiggyResult<Comment> {
    match comments::table.find(id).first(conn) {
      Ok(post) => Ok(post),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::PostNotFound)),
    }
  }

  pub fn author(&self, conn: &mut SqliteConnection) -> Option<User> {
    match User::by_id(conn, self.author_id) {
      Ok(user) => Some(user),
      Err(_) => None,
    }
  }

  pub fn get_all(conn: &mut SqliteConnection, article_id: i32) -> PiggyResult<Vec<Comment>> {
    match comments::table.filter(comments::article_id.eq(article_id)).load::<Comment>(conn) {
      Ok(comments) => Ok(comments),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::PostNotFound)),
    }
  }

  pub fn get_all_authors(conn: &mut SqliteConnection, article_id: i32) -> PiggyResult<HashMap<i32, User>> {
    let mut authors: HashMap<i32, User> = HashMap::new();
    for comment in Comment::get_all(conn, article_id)? {
      if !authors.contains_key(&comment.author_id) {
        authors.insert(comment.author_id, comment.author(conn).unwrap());
      }
    }
    Ok(authors)
  }

  pub fn by_author_id(conn: &mut SqliteConnection, author_id: i32) -> PiggyResult<Vec<Comment>> {
    match comments::table.filter(comments::author_id.eq(author_id)).load::<Comment>(conn) {
      Ok(comments) => Ok(comments),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::CommentNotFound)),
    }
  }
 
  pub fn delete(&self, conn: &mut SqliteConnection) -> PiggyResult<()> {
    diesel::delete(comments::table.find(self.id)).execute(conn)?;
    Ok(())
  }
}