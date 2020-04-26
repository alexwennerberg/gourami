-- Your SQL goes here

CREATE TABLE notes_temp (
id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
user_id INTEGER REFERENCES users(id),
content TEXT,
created_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO notes_temp(id,user_id,content,created_time)
SELECT id,creator_id,content,created_time
FROM notes;

drop table notes;
alter table notes_temp rename to notes;

alter table notes
add column in_reply_to references notes(id);

alter table users 
add column admin boolean default false;

CREATE TABLE notifications (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  notification_html TEXT,
  server_message BOOLEAN,
  created_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP 
);

CREATE TABLE notification_viewers (
  notification_id INTEGER REFERENCES notifications(id),
  user_id INTEGER REFERENCES users(id),
  viewed BOOLEAN,
  PRIMARY KEY (notification_id, user_id)
);

