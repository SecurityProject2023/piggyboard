-- Your SQL goes here
CREATE TABLE IF NOT EXISTS `comment_history` (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  comment_id INT NOT NULL,
  content TEXT NOT NULL,
  created_at DATETIME NOT NULL,
  who INT NOT NULL,
  FOREIGN KEY (comment_id) REFERENCES comments(id),
  FOREIGN KEY (who) REFERENCES users(id)
);