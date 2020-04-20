use serde_json::value::Value;

fn process_ap_activity(message: Value) {
    // profiles
// follow
// accept / reject

// statuses "Notes"
// match message.get("type").lower() {
//    "create" => Some(1),
   // "delete" => Some(1),
   // Announce?
    // _ => None
// };
// main type: Note
// simple https://docs.joinmastodon.org/spec/activitypub/
// support types:
// article
// page
// event
//
// "get" -> list all jsons. 
// match these to notifications
}
pub fn post_user_inbox(user_name: String, message: Value) {
}

pub fn post_user_outbox(user_name: String, message: Value) {
}

pub fn get_user_outbox(user_name: String) {
}

// requires authentication
pub fn get_user_inbox(user_name: String) {
}

pub fn user_followers(user_name: String) {
}

pub fn user_following(user_name: String) {
}
