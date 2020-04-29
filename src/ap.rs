/// Users don't follow users in Gourami. Instead the server does hte following
/// There are a number of reasons for this:
/// Gives it a more 'community' feel -- everyone shares the same timeline
/// Much simpler from an engineering and user perspective -- I think its difficult for
/// non-engineering people to properly separate different audience
///
/// This is a somewhat eccentric activitypub implementation, but it is as consistent with the spec
/// as I can make it!

use activitystreams::activity::{Accept, Activity, Announce, Create, Delete, Follow, Reject};
use activitystreams::BaseBox;
use log::debug;
use serde_json::{Value, Error};
use serde_json::from_str;
use crate::db::note::{RemoteNoteInput};

// gonna be big
//
// TODO -- use serde json here
fn process_unstructured_ap(message: &str) -> Result<(), Box<dyn std::error::Error>>{
    // Actions usually associated with notes
    // maybe there's a cleaner way to do this. cant iterate over types
    // TODO inbox forwarding https://www.w3.org/TR/activitypub/#inbox-forwarding
    let v: Value = serde_json::from_str(message)?;
    let _type = v.get("type").ok_or("No type found")?;
    if _type == "Create" {
        let object = v.get("object").ok_or("No object found")?;
        let _type = object.get("type").ok_or("No object type found")?;
        if _type == "Note" {
            let content = object.get("content").ok_or("No content found")?.as_str().ok_or("Not a string")?;
            // clean content 
            // let in_reply_to = match object.get("inReplyTo") {
            //     Some(v) => Some(v.as_str().ok_or("Not a string")?), // TODO -- get reply from database
                // None => None
            // };
            let remote_creator = object.get("attributedTo").ok_or("No attributedTo found")?.as_str().ok_or("Not a string")?;
            let remote_url = object.get("url").ok_or("No url Found")?.as_str().ok_or("Not a string")?;
            let remote_id = object.get("id").ok_or("No ID found")?.as_str().ok_or("Not a string")?;
            let new_remote_note = RemoteNoteInput {
            content: content,
            in_reply_to: None,
           neighborhood: true,
           is_remote: true,
           user_id: -1, // for remote. placeholder. not sure what to do with this ultimately
           remote_creator: remote_creator,
            remote_id: remote_id,
            remote_url: remote_url } ;
            println!("{:?}", new_remote_note);
        }
    }
    debug!("Unrecognized or invalid activity");
    Ok(())
}

fn new_note_to_ap_message() {
}

// /// used to send to others
// fn generate_ap(activity: Activity) {
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty_string() {
        process_unstructured_ap("{}");
    }

    #[test]
    fn test_mastodon_create_status_example() {
        let mastodon_create_note_json_string = r#"{
              "id": "https://mastodon.social/users/alexwennerberg/statuses/104028309437021899/activity",
              "type": "Create",
              "actor": "https://mastodon.social/users/alexwennerberg",
              "published": "2020-04-20T01:27:10Z",
              "to": [
                "https://www.w3.org/ns/activitystreams#Public"
              ],
              "cc": [
                "https://mastodon.social/users/alexwennerberg/followers"
              ],
              "object": {
                "id": "https://mastodon.social/users/alexwennerberg/statuses/104028309437021899",
                "type": "Note",
                "summary": null,
                "inReplyTo": null,
                "published": "2020-04-20T01:27:10Z",
                "url": "https://mastodon.social/@alexwennerberg/104028309437021899",
                "attributedTo": "https://mastodon.social/users/alexwennerberg",
                "to": [
                  "https://www.w3.org/ns/activitystreams#Public"
                ],
                "cc": [
                  "https://mastodon.social/users/alexwennerberg/followers"
                ],
                "sensitive": false,
                "atomUri": "https://mastodon.social/users/alexwennerberg/statuses/104028309437021899",
                "inReplyToAtomUri": null,
                "conversation": "tag:mastodon.social,2020-04-20:objectId=167583625:objectType=Conversation",
                "content": "<p>&lt;a href=&quot;<a href=\"https://google.com\" rel=\"nofollow noopener noreferrer\" target=\"_blank\"><span class=\"invisible\">https://</span><span class=\"\">google.com</span><span class=\"invisible\"></span></a>&quot;&gt;hi&lt;/a&gt;</p>",
                "contentMap": {
                  "en": "<p>&lt;a href=&quot;<a href=\"https://google.com\" rel=\"nofollow noopener noreferrer\" target=\"_blank\"><span class=\"invisible\">https://</span><span class=\"\">google.com</span><span class=\"invisible\"></span></a>&quot;&gt;hi&lt;/a&gt;</p>"
                },
                "attachment": [],
                "tag": [],
                "replies": {
                  "id": "https://mastodon.social/users/alexwennerberg/statuses/104028309437021899/replies",
                  "type": "Collection",
                  "first": {
                    "type": "CollectionPage",
                    "next": "https://mastodon.social/users/alexwennerberg/statuses/104028309437021899/replies?only_other_accounts=true&page=true",
                    "partOf": "https://mastodon.social/users/alexwennerberg/statuses/104028309437021899/replies",
                    "items": []
                  }
                }
              }
            }"#;
        process_unstructured_ap(mastodon_create_note_json_string);
    }
}

pub fn post_inbox(user_name: String, message: Value) {}

pub fn post_outbox(user_name: String, message: Value) {}

// TODO figure out how to follow mastodon
//
pub fn user_followers(user_name: String) {}

pub fn user_following(user_name: String) {}
