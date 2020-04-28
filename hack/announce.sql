insert into notifications (notification_html, server_message)
values("New feature alert! Added notifications, made a few other minor tweaks", true);
-- TODO find a better way 
insert into notification_viewers (notification_id, user_id, viewed)
select 6,id,false from users;
