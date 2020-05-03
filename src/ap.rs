use crate::db::conn::POOL;
use crate::db::note::{NoteInput, RemoteNoteInput};
use crate::db::user::{NewRemoteUser, User};
use diesel::insert_into;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde_json::json;
use serde_json::Value;
use std::env;
/// Users don't follow users in Gourami. Instead the server does hte following
/// There are a number of reasons for this:
/// Gives it a more 'community' feel -- everyone shares the same timeline
/// Much simpler from an engineering and user perspective -- I think its difficult for
/// non-engineering people to properly separate different audience
///
/// This is a somewhat eccentric activitypub implementation, but it is as consistent with the spec
/// as I can make it!
use std::fs;
lazy_static! {
    // const SERVER_ACTOR = "gourami.social"
}

// ActivityPub outbox
fn send_to_outbox(activity: bool) {
    // activitystreams object fetch/store from db.  db objects need to serialize/deserialize this object if get -> fetch from db if post -> put to db, send to inbox of followers send to inbox of followers
}

enum Action {
    CreateNote,
    DoNothing,
    // DeleteNote
}

/// get the server user json
fn server_actor() -> Value {
    let domain = env::var("GOURAMI_DOMAIN").unwrap();
    let actor = format!("{}/actor", domain);
    let inbox = format!("{}/inbox", domain);
    let public_key = fs::read_to_string(env::var("SIGNATURE_PUBKEY").unwrap()).unwrap();
    json!({
    "@context": [
        "https://www.w3.org/ns/activitystreams",
        "https://w3id.org/security/v1"
    ],

    "id": actor,
    "type": "Organization", // application?
    "preferredUsername": domain, // think about it
    "inbox": inbox,
    "publicKey": {
        "id": format!("{}#main-key", actor),
        "owner": actor,
        "publicKeyPem": public_key
    }})
}

fn categorize_input_message(v: Value) -> Action {
    Action::DoNothing
}

pub fn process_create_note(
    conn: &SqliteConnection,
    v: Value,
) -> Result<(), Box<dyn std::error::Error>> {
    // Actions usually associated with notes
    // maybe there's a cleaner way to do this. cant iterate over types
    // TODO inbox forwarding https://www.w3.org/TR/activitypub/#inbox-forwarding
    let object = v.get("object").ok_or("No object found")?;
    let _type = object.get("type").ok_or("No object type found")?;
    let content = object
        .get("content")
        .ok_or("No content found")?
        .as_str()
        .ok_or("Not a string")?;
    // clean content
    // let in_reply_to = match object.get("inReplyTo") {
    //     Some(v) => Some(v.as_str().ok_or("Not a string")?), // TODO -- get reply from database
    // None => None
    // };
    let remote_creator = object
        .get("attributedTo")
        .ok_or("No attributedTo found")?
        .as_str()
        .ok_or("Not a string")?;
    let remote_url = object
        .get("url")
        .ok_or("No url Found")?
        .as_str()
        .ok_or("Not a string")?;
    let remote_id = object
        .get("id")
        .ok_or("No ID found")?
        .as_str()
        .ok_or("Not a string")?;

    use crate::db::schema::notes::dsl as n;
    use crate::db::schema::users::dsl as u;
    //  if user not in db, insert
    //
    let new_user = NewRemoteUser {
        username: String::from(remote_creator),
        remote_url: Some(String::from(remote_creator)),
    };
    //
    insert_into(u::users).values(&new_user).execute(conn).ok(); // TODO only check unique constraint error

    let new_user_id: i32 = u::users
        .select(u::id)
        .filter(u::remote_url.eq(remote_creator))
        .first(conn)
        .unwrap();

    let new_remote_note = RemoteNoteInput {
        content: String::from(content),
        in_reply_to: None, // TODO
        neighborhood: true,
        is_remote: true,
        user_id: new_user_id, // for remote. placeholder. not sure what to do with this ultimately
        remote_creator: String::from(remote_creator),
        remote_id: String::from(remote_id),
        remote_url: String::from(remote_url),
    };
    println!("{:?}", new_remote_note);
    insert_into(n::notes)
        .values(&new_remote_note)
        .execute(conn)
        .unwrap();
    return Ok(());
}

pub fn get_destinations() -> Vec<String> {
    // maybe lazy static this
    use crate::db::schema::server_mutuals::dsl::*;
    let conn = &POOL.get().unwrap();
    server_mutuals.select(inbox_url).load(conn).unwrap()
}

pub async fn send_ap_message(
    ap_message: &Value,
    destinations: Vec<String>,
) -> Result<(), reqwest::Error> {
    // Right now we have only once delivery
    for destination in destinations {
        let client = reqwest::Client::new();
        client.post(&destination).json(&ap_message).send().await?;
    }
    Ok(())
}

fn follow_remote_server(remote_url: String) {
    // create follow request
}

fn generate_server_follow(remote_url: String) -> Value {
    json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": "https://my-example.com/my-first-follow",
        "type": "Follow",
        "actor": "https://my-example.com/actor",
        "object": remote_url,
    })
}

/// Generate an AP create message from a new note
pub fn new_note_to_ap_message(note: &NoteInput, user: &User) -> Value {
    // we need note, user. note noteinput but note obj
    let conn = &POOL.get().unwrap();
    // Do a bunch of db queries to get the info I need
    json!({
        "@context": "https://www.w3.org/ns/activitystreams",
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
            "content": note.content
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

    // #[test]
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
        assert_eq!(
            process_create_note(create_note_mastodon).unwrap(),
            RemoteNoteInput {
                content: String::from("hello world"),
                in_reply_to: None,
                neighborhood: true,
                is_remote: true,
                user_id: -1, // for remote. placeholder. not sure what to do with this ultimately
                remote_creator: String::from("https://mastodon.social/users/alexwennerberg"),
                remote_id: String::from(
                    "https://mastodon.social/users/alexwennerberg/statuses/104028309437021899"
                ),
                remote_url: String::from(
                    "https://mastodon.social/@alexwennerberg/104028309437021899"
                ),
            }
        )
    }
}
