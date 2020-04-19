-- Your SQL goes here

CREATE TABLE users (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  username VARCHAR(255),
  password VARCHAR(255),
  email VARCHAR(255)
);

CREATE UNIQUE INDEX users_username_idx ON users (username);

CREATE TABLE activities (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  json_text TEXT
);

CREATE TABLE sessions (
  id INTEGER NOT NULL  PRIMARY KEY AUTOINCREMENT,
  cookie VARCHAR NOT NULL,
  user_id INTEGER NOT NULL REFERENCES users (id)
);

-- media_attachments

CREATE TABLE notes (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    creator_id INTEGER,
    parent_id INTEGER,
    content TEXT,
    published TEXT
);

