mod kind;
use actix_web::{ResponseError, HttpResponse};
pub use kind::*;
// use serde_json::json;
use std::error::Error;
use diesel::result::Error as DieselError;

use crate::res::error::ErrorResponse;

#[derive(Debug)]
pub struct PiggyError {
  pub details: String,
  pub kind: PiggyErrorKind
}

impl PiggyError {
  pub fn new(msg: impl ToString, kind: PiggyErrorKind) -> PiggyError {
    PiggyError{ details: msg.to_string(), kind }
  }

  pub fn from_kind(kind: PiggyErrorKind) -> PiggyError {
    let kindstr: String = kind.to_string();
    PiggyError{
      details: kindstr.split('_').map(|w| {
        let mut chars: std::str::Chars<'_> = w.chars();
        match chars.next() {
          None => String::new(),
          Some(first_char) => format!("{}{}", first_char.to_uppercase(), chars.as_str().to_lowercase())
        }
      }).collect::<Vec<String>>().join(" "),
      kind
    }
  }
}

impl Error for PiggyError {}

impl ResponseError for PiggyError {
  fn error_response(&self) -> HttpResponse {
    match self.kind {
      _ => ErrorResponse::new_b(self).build(&mut HttpResponse::BadRequest()),
    }
  }
}

impl From<DieselError> for PiggyError {
  fn from(value: DieselError) -> Self {
    match value { _ => PiggyError::new(value.to_string(), PiggyErrorKind::DieselError) }
  }
}

impl std::fmt::Display for PiggyError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f,"PiggyError {}: {}", self.kind as i32, self.details)
  }
}