-- Your SQL goes here

CREATE TABLE user (
id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  username VARCHAR(255),
  email VARCHAR(255),
  created_at TEXT,
  private_key TEXT,
  public_key TEXT
);

CREATE TABLE notification (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT
);

-- media_attachments

CREATE TABLE status (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    creator_id INTEGER,
    parent_id INTEGER,
    content TEXT,
    published TEXT
);

