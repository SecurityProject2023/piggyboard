pub enum Acl {
  Ban,
  All,
  User,
  Email,
  Author,
  Admin,
  Owner,
}

impl From<&str> for Acl {
  fn from(s: &str) -> Self {
    match s {
      "ban" => Acl::Ban,
      "all" => Acl::All,
      "user" => Acl::User,
      "email" => Acl::Email,
      "author" => Acl::Author,
      "admin" => Acl::Admin,
      "owner" => Acl::Owner,
      _ => Acl::All,
    }
  }
}

impl From<String> for Acl {
  fn from(s: String) -> Self {
    match s.as_str() {
      "ban" => Acl::Ban,
      "all" => Acl::All,
      "user" => Acl::User,
      "email" => Acl::Email,
      "author" => Acl::Author,
      "admin" => Acl::Admin,
      "owner" => Acl::Owner,
      _ => Acl::All,
    }
  }
}