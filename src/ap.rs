use log::{debug};
use serde_json::{Value};
use activitystreams::activity::{Create, Accept, Follow, Reject, Announce, Delete, Activity};

// gonna be big
fn process_unstructured_ap(message: &str) {
    // Actions usually associated with notes
    use serde_json::from_str;
    // maybe there's a cleaner way to do this. cant iterate over types
    // TODO inbox forwarding https://www.w3.org/TR/activitypub/#inbox-forwarding
    if let Some(create) = from_str::<Create>(message).ok() {
        // create note database object
    }
    else if let Some(delete) = from_str::<Delete>(message).ok() {
        // delete note database object
    }
    debug!("Unrecognized or invalid activity");
}

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
