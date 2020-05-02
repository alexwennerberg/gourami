-- Your SQL goes here

alter table users add remote_url VARCHAR(1000);

create unique index uniqremote on users(remote_url);

create table server_mutuals (
  id integer primary key autoincrement,
  inbox_url VARCHAR(1000),
  outbox_url VARCHAR(1000)
);

