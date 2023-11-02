use crate::error::PiggyError;
use actix_web::{HttpResponseBuilder, HttpResponse};
use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
  pub code: i32,
  pub error_type: String,
  pub message: String
}

impl ErrorResponse {
  pub fn new(error: PiggyError) -> ErrorResponse{
    ErrorResponse {
      code: error.kind as i32,
      error_type: error.kind.to_string(),
      message: error.to_string()
    }
  }

  pub fn new_b(error: &PiggyError) -> ErrorResponse{
    ErrorResponse {
      code: error.kind as i32,
      error_type: error.kind.to_string(),
      message: error.to_string()
    }
  }

  pub fn build(&self, builder: &mut HttpResponseBuilder) -> HttpResponse {
    builder.json(json!({"error":&self}))
  }
}

impl actix_web::ResponseError for ErrorResponse {
  fn error_response(&self) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({"error":&self}))
  }
}

impl std::fmt::Display for ErrorResponse {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f,"ErrorResponse {}: {}", self.code, self.message)
  }
}