/// Users don't follow users in Gourami. Instead the server does hte following
/// There are a number of reasons for this:
/// Gives it a more 'community' feel -- everyone shares the same timeline
/// Much simpler from an engineering and user perspective -- I think its difficult for
/// non-engineering people to properly separate different audience
///
/// This is a somewhat eccentric activitypub implementation, but it is as consistent with the spec
/// as I can make it!

use serde_json::json;
use serde_json::{Value};
use crate::db::note::{NoteInput, RemoteNoteInput};
use warp::{Reply, Filter, Rejection};


lazy_static! {
    // const SERVER_ACTOR = "gourami.social"
}

enum Action {
    CreateNote,
    DoNothing,
    // DeleteNote
}


fn categorize_input_message(v: Value) -> Action {
    Action::DoNothing
}

pub fn parse_create_note(v: Value) -> Result<RemoteNoteInput, Box<dyn std::error::Error>>{
    // Actions usually associated with notes
    // maybe there's a cleaner way to do this. cant iterate over types
    // TODO inbox forwarding https://www.w3.org/TR/activitypub/#inbox-forwarding
    let object = v.get("object").ok_or("No object found")?;
    let _type = object.get("type").ok_or("No object type found")?;
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
    content: String::from(content),
    in_reply_to: None,
    neighborhood: true,
    is_remote: true,
    user_id: -1, // for remote. placeholder. not sure what to do with this ultimately
    remote_creator: String::from(remote_creator),
    remote_id: String::from(remote_id),
    remote_url: String::from(remote_url) } ;
    println!("{:?}", new_remote_note);
    return Ok(new_remote_note);
}


/// Generate an AP create message from a new note
pub fn new_note_to_ap_message(note_input: &NoteInput) -> Value{
    json!({
        "id": "someid",
        "type": "Create",
        "actor": "my_server/actor", // get from DEPLOY_URL
        "published": "now",
        "to": [
            "destination.server"
        ],
        "object": {
            "id": "unique id",
            "type": "note",
            "url": "abc",
            "inReplyTo": "none",
            "attributedTo": "joe",
            "content": "whats up",
        }
    })
}

// /// used to send to others
// fn generate_ap(activity: Activity) {
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty_string() {
        // to write
    }

    #[test]
    fn test_mastodon_create_status_example() {
        let create_note_mastodon = serde_json::from_str(r#"{
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
                "content": "hello world",
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
            }"#).unwrap();
        assert_eq!(parse_create_note(create_note_mastodon).unwrap(), RemoteNoteInput {
            content: String::from("hello world"),
        in_reply_to: None,
        neighborhood: true,
        is_remote: true,
        user_id: -1, // for remote. placeholder. not sure what to do with this ultimately
        remote_creator: String::from("https://mastodon.social/users/alexwennerberg"),
        remote_id: String::from("https://mastodon.social/users/alexwennerberg/statuses/104028309437021899"),
        remote_url: String::from("https://mastodon.social/@alexwennerberg/104028309437021899"),
        })
    }
}
