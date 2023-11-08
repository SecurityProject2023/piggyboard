-- Your SQL goes here
CREATE TABLE IF NOT EXISTS `users` (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  profile_img TEXT NOT NULL DEFAULT 'https://static.fishydi.no/img/defaultProfileImg.png',
  profile_thumbnail TEXT NOT NULL DEFAULT 'https://static.fishydi.no/img/defaultProfileThumbnail.png',
  profile_banner TEXT NOT NULL DEFAULT 'https://static.fishydi.no/img/defaultProfileBanner.png',
  first_name TEXT NOT NULL,
  last_name TEXT DEFAULT '',
  gender TEXT NOT NULL,
  username TEXT NOT NULL UNIQUE,
  email TEXT NOT NULL UNIQUE,
  phone TEXT DEFAULT NULL,
  lang TEXT NOT NULL DEFAULT 'en',
  bio TEXT NOT NULL DEFAULT '',
  password TEXT NOT NULL,
  verified BOOLEAN NOT NULL DEFAULT 0,
  acl TEXT NOT NULL,
  created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);