use log::{debug, info};
use serde_json::{Result, Value};
use std::error::Error;
use activitystreams::activity::{Create, Activity};

fn parse_unstructured_ap(message: &str) -> impl Activity {
    // try and serialize in a few different ways
    let object: Create = serde_json::from_str(message).unwrap();
    object
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty_string() {
        parse_unstructured_ap("1");
    }
}


    // Err("Message not an activity");
// let object = message.get("object")?.get("type")?;
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
