use argon2::{
  password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
  Argon2
};
use md5::{Md5, Digest};
use crate::{PiggyResult, error::PiggyError};

pub fn password_hash(password: &str) -> PiggyResult<String>{
  let password: &[u8] = password.as_bytes();
  let salt: SaltString = SaltString::generate(&mut OsRng);
  let argon2: Argon2<'_> = Argon2::default();
  match argon2.hash_password(password, &salt) {
    Ok(hash) => Ok(hash.to_string()),
    Err(e) => Err(PiggyError::new(e.to_string(), crate::error::PiggyErrorKind::PasswordHashCreateFailed)),
  }
}

pub fn verify_password(hash: &str, password: &str) -> PiggyResult<bool> {
  let password: &[u8] = password.as_bytes();
  let parsed_hash: PasswordHash<'_> = match PasswordHash::new(hash) {
    Ok(hash) => hash,
    Err(e) => return Err(PiggyError::new(e.to_string(), crate::error::PiggyErrorKind::PasswordHashCreateFailed)),
  };
  Ok(Argon2::default().verify_password(password, &parsed_hash).is_ok())
}

pub fn md5(input: &impl ToString) -> String {
  let mut hasher = Md5::new();
  hasher.update(input.to_string());
  let result = hasher.finalize();
  format!("{:x}", result)
}