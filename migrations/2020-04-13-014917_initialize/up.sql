-- Your SQL goes here

CREATE TABLE user (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  username VARCHAR(255),
  email VARCHAR(255),
  created_at TEXT,
  private_key TEXT,
  public_key TEXT
);

CREATE TABLE activity (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  json_text TEXT
);

-- media_attachments

CREATE TABLE note (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    creator_id INTEGER,
    parent_id INTEGER,
    content TEXT,
    published TEXT
);

