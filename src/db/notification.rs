use super::schema::{notification_viewers, notifications};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Clone, Deserialize, Serialize)]
pub struct Notification {
    // rename RenderedNote
    pub id: i32,
    pub notification_html: String,
    pub server_message: bool, // messages sent to everyone. maybe not necc
    pub created_time: String,
}

#[derive(Queryable, Clone, Deserialize, Serialize)]
pub struct NotificationViewer {
    // rename RenderedNote
    pub notification_id: i32,
    pub user_id: i32,
    pub viewed: bool,
}

#[derive(Insertable)]
#[table_name = "notifications"]
pub struct NewNotification {
    // rename RenderedNote
    pub notification_html: String,
    pub server_message: bool, // messages sent to everyone. maybe not necc
}

#[derive(Insertable)]
#[table_name = "notification_viewers"]
pub struct NewNotificationViewer {
    // rename RenderedNote
    pub notification_id: i32,
    pub user_id: i32,
    pub viewed: bool,
}
