use diesel::prelude::*;
use crate::{schema::article_acl, PiggyResult, error::{PiggyError, PiggyErrorKind}};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AclData {
  pub pread: i32,
  pub pedit: i32,
  pub pdelete: i32,
  pub prate: i32,
  pub pread_rate: i32,
  pub prv: i32,
  pub pwd: i32,
  pub ped: i32,
  pub pview_history: i32,
  pub pread_comments: i32,
  pub pwrite_comments: i32,
  pub pedit_comments: i32,
  pub pdelete_comments: i32,
}

#[derive(Queryable, Debug, PartialEq, Serialize, AsChangeset)]
#[diesel(table_name = article_acl)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ArticleAcl {
  pub id: i32,
  pub article_id: i32,
  pub pread: i32,
  pub pedit: i32,
  pub pdelete: i32,
  pub prate: i32,
  pub pread_rate: i32,
  pub prv: i32,
  pub pwd: i32,
  pub ped: i32,
  pub pview_history: i32,
  pub pread_comments: i32,
  pub pwrite_comments: i32,
  pub pedit_comments: i32,
  pub pdelete_comments: i32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = article_acl)]
pub struct NewArticleAcl {
  pub article_id: i32,
  pub pread: i32,
  pub pedit: i32,
  pub pdelete: i32,
  pub prate: i32,
  pub pread_rate: i32,
  pub prv: i32,
  pub pwd: i32,
  pub ped: i32,
  pub pview_history: i32,
  pub pread_comments: i32,
  pub pwrite_comments: i32,
  pub pedit_comments: i32,
  pub pdelete_comments: i32,
}

impl Default for NewArticleAcl {
  fn default() -> Self {
    NewArticleAcl {
      article_id: 0,
      pread: 1,
      pedit: 4,
      pdelete: 4,
      prate: 2,
      pread_rate: 1,
      prv: 1,
      pwd: 1,
      ped: 1,
      pview_history: 5,
      pread_comments: 1,
      pwrite_comments: 2,
      pedit_comments: 4,
      pdelete_comments: 4,
    }
  }
}

impl ArticleAcl {
  pub fn new(article_id: i32) -> NewArticleAcl {
    NewArticleAcl {
      article_id,
      ..Default::default()
    }
  }

  pub fn add(conn: &mut SqliteConnection, post: NewArticleAcl) -> PiggyResult<ArticleAcl> {
    diesel::insert_into(article_acl::table)
      .values(&post)
      .execute(conn)?;
    Ok(article_acl::table.order(article_acl::id.desc()).first(conn)?)
  }

  pub fn update(mut self, conn: &mut SqliteConnection, post: AclData) -> PiggyResult<ArticleAcl> {
    self.pread = post.pread;
    self.pedit = post.pedit;
    self.pdelete = post.pdelete;
    self.prate = post.prate;
    self.pread_rate = post.pread_rate;
    self.prv = post.prv;
    self.pwd = post.pwd;
    self.ped = post.ped;
    self.pview_history = post.pview_history;
    self.pread_comments = post.pread_comments;
    self.pwrite_comments = post.pwrite_comments;
    self.pedit_comments = post.pedit_comments;
    self.pdelete_comments = post.pdelete_comments;
    let id = self.id;
    diesel::update(article_acl::table.find(self.id))
      .set(self)
      .execute(conn)?;
    Ok(article_acl::table.find(id).first(conn)?)
  }

  pub fn by_id(conn: &mut SqliteConnection, id: i32) -> PiggyResult<ArticleAcl> {
    match article_acl::table.find(id).first(conn) {
      Ok(post) => Ok(post),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::PostNotFound)),
    }
  }

  pub fn by_article_id(conn: &mut SqliteConnection, article_id: i32) -> PiggyResult<ArticleAcl> {
    match article_acl::table.filter(article_acl::article_id.eq(article_id)).first(conn) {
      Ok(post) => Ok(post),
      Err(_) => Err(PiggyError::from_kind(PiggyErrorKind::PostNotFound)),
    }
  }
}