use std::collections::HashMap;

use diesel::prelude::*;
use crate::{schema::articles, PiggyResult, error::{PiggyError, PiggyErrorKind}};
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;

use super::{User, Comment};

#[derive(Queryable, Debug, PartialEq, Serialize)]
#[diesel(table_name = article)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Article {
  pub id: i32,
  pub author_id: i32,
  pub title: String,
  pub content: String,
  pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = articles)]
pub struct NewArticle {
  pub author_id: i32,
  pub title: String,
  pub content: String,
  pub created_at: NaiveDateTime,
}

impl Article {
  pub fn new(author_id: i32, title: String, content: String) -> NewArticle {
    NewArticle {
      author_id,
      title,
      content,
      created_at: Utc::now().naive_utc(),
    }
  }

  pub fn add(conn: &mut SqliteConnection, post: NewArticle) -> PiggyResult<Article> {
    diesel::insert_into(articles::table)
      .values(&post)
      .execute(conn)?;
    Ok(articles::table.order(articles::id.desc()).first(conn)?)
  }

  pub fn by_id(conn: &mut SqliteConnection, id: i32) -> PiggyResult<Article> {
    match articles::table.find(id).first(conn) {
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

  pub fn get_all(conn: &mut SqliteConnection) -> PiggyResult<Vec<Article>> { Ok(articles::table.load::<Article>(conn)?) }

  pub fn get_all_comments(&self, conn: &mut SqliteConnection) -> PiggyResult<Vec<Comment>> {
    match Comment::get_all(conn, self.id) {
      Ok(comments) => Ok(comments),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::CommentNotFound)),
    }
  }

  pub fn get_all_comment_authors(&self, conn: &mut SqliteConnection) -> PiggyResult<HashMap<i32, User>> {
    match Comment::get_all_authors(conn, self.id) {
      Ok(authors) => Ok(authors),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::CommentNotFound)),
    }
  }

  pub fn by_author_id(conn: &mut SqliteConnection, author_id: i32) -> PiggyResult<Vec<Article>> {
    match articles::table.filter(articles::author_id.eq(author_id)).load::<Article>(conn) {
      Ok(posts) => Ok(posts),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::PostNotFound)),
    }
  }
  
  pub fn delete(&self, conn: &mut SqliteConnection) -> PiggyResult<()> {
    diesel::delete(articles::table.find(self.id)).execute(conn)?;
    Ok(())
  }
}