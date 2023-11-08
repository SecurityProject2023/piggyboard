-- Your SQL goes here
CREATE TABLE IF NOT EXISTS `article_history` (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  article_id INT NOT NULL,
  title TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at DATETIME NOT NULL,
  who INT NOT NULL,
  FOREIGN KEY (article_id) REFERENCES articles(id),
  FOREIGN KEY (who) REFERENCES users(id)
);