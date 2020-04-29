-- Your SQL goes here

alter table notes add is_remote boolean default false;
alter table notes add remote_url varchar(1000); -- semi-arbitrary
alter table notes add remote_creator varchar(1000); -- semi-arbitrary
alter table notes add remote_id varchar(1000); -- semi-arbitrary;
