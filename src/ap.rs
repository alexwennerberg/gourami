use crate::db::conn::POOL;
use crate::db::note::{Note, NoteInput, RemoteNoteInput};
use crate::db::server_mutuals::{NewServerMutual, ServerMutual};
use crate::db::user::{NewRemoteUser, User};
use crate::error::Error;
use base64;
use chrono::{Duration, Utc};
use diesel::insert_into;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use http_signature_normalization::Config;
use openssl::rsa::Rsa;
use reqwest::Request;
use ring::digest;
use ring::signature::UnparsedPublicKey;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::path::Path;

fn domain_url() -> String {
    if &env::var("SSL_ENABLED").unwrap() == "1" {
        return format!("https://{}", &env::var("GOURAMI_DOMAIN").unwrap());
    }
    return format!("http://{}", &env::var("GOURAMI_DOMAIN").unwrap());
}

pub struct ServerApData {
    pub global_id: String,
    pub key_id: String,
    pub domain: String,
    pub inbox: String,
    pub public_key: String,
}

lazy_static! {
    // TODO -- learn this a little better so it isnt so redundant
    pub static ref SERVER: ServerApData = ServerApData {
        global_id: format!("{}", domain_url()),
        domain: env::var("GOURAMI_DOMAIN").unwrap(),
        key_id: format!("{}#key", domain_url()),
        inbox: format!("{}/inbox", domain_url()),
        public_key: std::fs::read_to_string(env::var("SIGNATURE_PUBKEY_PEM").unwrap()).unwrap()
    };
}

// TODO figure out how to get static working

use uuid::Uuid;

fn generate_activity_id() -> String {
    let my_uuid = Uuid::new_v4();
    format!("{}/activity/{}", domain_url(), my_uuid)
}

#[derive(Deserialize, Serialize)]
pub struct CreateNote {
    // Maybe use AP crate
    id: String,
    note: ApNote,
    actor: Actor,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Actor {
    #[serde(rename = "@context")]
    context: Value,
    id: String,
    name: Option<String>,
    summary: Option<String>,
    #[serde(rename = "type")]
    _type: String,
    #[serde(rename = "preferredUsername")]
    preferred_username: String,
    inbox: String,
    #[serde(rename = "publicKey")]
    public_key: PublicKey,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApNote {
    content: String,
    #[serde(rename = "attributedTo")]
    attributed_to: String,
    url: String,
    summary: Option<String>,
    id: String,
    #[serde(rename = "inReplyTo")]
    in_reply_to: Option<String>,
}

use regex::Regex;

impl ApNote {
    fn get_remote_user_name(&self) -> Option<String> {
        let re = Regex::new(r"^(.+?)(💬)").unwrap();
        match re.captures(&self.content) {
            Some(t) => t.get(1).unwrap().as_str().parse().ok(),
            None => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PublicKey {
    id: String,
    owner: String,
    #[serde(rename = "publicKeyPem")]
    public_key_pem: String,
}

/// Users don't follow users in Gourami. Instead the server does hte following
/// There are a number of reasons for this:
/// Gives it a more 'community' feel -- everyone shares the same timeline
/// Much simpler from an engineering and user perspective -- I think its difficult for
/// non-engineering people to properly separate different audience
///
/// This is a somewhat eccentric activitypub implementation, but it is as consistent with the spec
/// as I can make it!
use std::fs;
// ActivityPub outbox
fn send_to_outbox(activity: bool) {
    // activitystreams object fetch/store from db.  db objects need to serialize/deserialize this object if get -> fetch from db if post -> put to db, send to inbox of followers send to inbox of followers
}

#[derive(Deserialize)]
pub struct WebFingerQuery {
    resource: String,
}

pub fn webfinger_json(query: WebFingerQuery) -> Value {
    // global -- single user
    json!({
      "aliases": [
        SERVER.global_id
      ],
      "links": [
        {
          "href": SERVER.global_id,
          "rel": "self",
          "type": "application/activity+json"
        }
      ],
      "subject": format!("acct:server@{}", SERVER.domain),
    })
}

/// get the server user json
pub fn server_actor_json() -> Actor {
    // TODO figure out how to get lazy static working
    // TODO use ap library
    serde_json::from_value(json!({
    "@context": [
        "https://www.w3.org/ns/activitystreams",
        "https://w3id.org/security/v1"
    ],
    "id": SERVER.global_id,
    "type": "Organization", // application?
    "preferredUsername": domain_url(), // think about it
    "inbox": SERVER.inbox,
    "name": "server",
    "summary": "server",
    "publicKey": {
        "id": SERVER.key_id,
        "owner": SERVER.global_id,
        "publicKeyPem": SERVER.public_key
    // TODO -- list server admin contact somewhere. summary or attachment
    }}))
    .unwrap()
}

pub fn process_create_note(v: Value) -> Result<(), Box<dyn std::error::Error>> {
    // Actions usually associated with notes
    // maybe there's a cleaner way to do this. cant iterate over types
    // TODO inbox forwarding https://www.w3.org/TR/activitypub/#inbox-forwarding
    //
    let conn = &POOL.get()?;
    // Get actor
    // TODO -- look into this
    let object = v.get("object").ok_or("No AP object found")?;
    let _type = object.get("type").ok_or("No object type found")?;
    // match type == note
    let ap_note: ApNote = serde_json::from_value(object.to_owned())?;

    use crate::db::schema::notes::dsl as n;
    use crate::db::schema::users::dsl as u;
    //  if user not in db, insert
    //
    let remote_username = ap_note
        .get_remote_user_name()
        .unwrap_or(ap_note.attributed_to); // TODO -- prevent usernames iwth colons
                                           // strip out username
    let new_user = NewRemoteUser {
        username: remote_username.clone(),
        remote_user: true,
    };

    let new_user_id: i32 = conn.transaction(|| {
        insert_into(u::users).values(&new_user).execute(conn).ok(); // TODO only check unique constraint error
                                                                    // last insert id
        u::users
            .select(u::id)
            .filter(u::username.eq(&remote_username))
            .first(conn)
    })?;

    let new_remote_note = RemoteNoteInput {
        content: ap_note.content,
        in_reply_to: None, // TODO
        neighborhood: true,
        is_remote: true,
        user_id: new_user_id,
        remote_id: ap_note.id,
        remote_url: ap_note.url,
    };
    insert_into(n::notes)
        .values(&new_remote_note)
        .execute(conn)?;
    return Ok(());
}

pub async fn process_accept(v: Value) -> Result<(), Error> {
    let actor_id: &str = v
        .get("actor")
        .ok_or("No actor found")?
        .as_str()
        .ok_or("Not a string")?;
    set_mutual_accepted(actor_id);
    Ok(())
}

fn set_mutual_accepted(the_actor_id: &str) -> Result<(), Error> {
    use crate::db::schema::server_mutuals::dsl::*;
    let conn = &POOL.get()?;
    diesel::update(server_mutuals)
        .filter(actor_id.eq(the_actor_id))
        .set(accepted.eq(true))
        .execute(conn)?;
    Ok(())
}

// TODO clean this up
fn set_mutual_followed_back(the_actor_id: &str) -> Result<(), Error> {
    use crate::db::schema::server_mutuals::dsl::*;
    let conn = &POOL.get()?;
    diesel::update(server_mutuals)
        .filter(actor_id.eq(the_actor_id))
        .set(followed_back.eq(true))
        .execute(conn)?;
    Ok(())
}

fn should_accept(actor_id: &str) -> bool {
    use crate::db::schema::server_mutuals::dsl as s;
    let conn = &POOL.get().unwrap();
    let sent_req: bool = s::server_mutuals
        .select(s::actor_id)
        .filter(s::actor_id.eq(actor_id))
        .first::<String>(conn)
        .is_ok();
    sent_req
}

pub async fn process_follow(v: Value) -> Result<(), Error> {
    let actor: &str = v.get("actor").unwrap().as_str().unwrap();
    let remote_actor: Actor = get_remote_actor(actor).await?; // not strictly necessary can use db instead
    let actor_inbox = &remote_actor.inbox;
    let sent_req = true; // should_accept(actor);
    if sent_req {
        set_mutual_followed_back(actor)?;
        // send accept follow
        let accept = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": generate_activity_id(),
        "type": "Accept",
        "actor": SERVER.global_id,
        "object": &v,
        });
        send_ap_message(&accept, actor_inbox.to_string())
            .await
            .unwrap();
    }
    Ok(())
    // generate accept
}

pub fn get_connected_remotes() -> Vec<ServerMutual> {
    // maybe lazy static this
    use crate::db::schema::server_mutuals::dsl::*;
    let conn = &POOL.get().unwrap();
    server_mutuals
        .filter(accepted.eq(true))
        .filter(followed_back.eq(true))
        .load(conn)
        .unwrap()
}

pub async fn send_ap_message(
    ap_message: &Value,
    destination: String, // really vec of URLs
) -> Result<(), Error> {
    debug!("Sending outgoing AP message to {}", destination);
    let msg = Vec::from(ap_message.to_string().as_bytes());
    let client = reqwest::Client::new();
    let response = client
        .post(&destination)
        .header(
            "date",
            Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
        ) //HTTP time format
        .body(msg)
        .header(
            "Content-Type",
            r#"application/ld+json; profile="https://www.w3.org/ns/activitystreams""#,
        )
        .http_sign_outgoing()?
        .send()
        .await?;
    debug!("{:?}", response.text().await?);
    Ok(())
}
pub async fn get_remote_actor(actor_id: &str) -> Result<Actor, Error> {
    debug!("Fetching remote actor {}", actor_id);
    let client = reqwest::Client::new();
    let res = client
        .get(actor_id)
        .header(
            "Accept",
            r#"application/ld+json; profile="https://www.w3.org/ns/activitystreams""#,
        )
        .send()
        .await?;
    let res: Actor = res.json().await?;
    Ok(res)
}

// TODO cleanup interface
pub async fn whitelist_or_follow_remote_server(remote_url: &str, send: bool) -> Result<(), Error> {
    let remote_actor: Actor = get_remote_actor(remote_url).await?;
    let inbox_url = &remote_actor.inbox;
    let actor_id = &remote_actor.id;
    let msg = generate_server_follow(actor_id, inbox_url)?;
    if send {
        send_ap_message(&msg, inbox_url.to_owned()).await?;
    }
    Ok(())
}

fn generate_server_follow(remote_actor: &str, my_inbox_url: &str) -> Result<Value, Error> {
    let conn = &POOL.get()?;
    let res = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": generate_activity_id(),
        "type": "Follow",
        "actor": SERVER.global_id,
        "object": remote_actor,
    });
    use crate::db::schema::server_mutuals::dsl::*;
    // TODO use str instead of String
    insert_into(server_mutuals)
        .values(NewServerMutual {
            actor_id: remote_actor.to_owned(),
            inbox_url: my_inbox_url.to_owned(),
        })
        .execute(conn)?;
    Ok(res)
}

/// Generate an AP create message from a new note
pub fn new_note_to_ap_message(note: &Note, user: &User) -> Value {
    // we need note, user. note noteinput but note obj
    // Do a bunch of db queries to get the info I need
    //
    // prepend the username to the content
    // strip it out on receipt
    // use a field separator
    let content = note.get_content_for_outgoing(&user.username);
    json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "id": generate_activity_id(),
        "type": "Create",
        "actor": SERVER.global_id,
        "published": note.created_time, // doesnt match
        "to": [
            "https://www.w3.org/ns/activitystreams#Public"
        ], // todo audience
        "object": {
            "id": note.get_url(), // TODO generate
            "type": "Note",
            "summary": "", // unused
            "url": note.get_url(),
            "attributedTo": SERVER.global_id,
            "content": content
        }
    })
}

// /// used to send to others
// fn generate_ap(activity: Activity) {
// }
pub trait HttpSignature {
    fn http_sign_outgoing(self) -> Result<reqwest::RequestBuilder, Error>;
}

// fn read_file(path: &std::path::Path) -> Vec<u8> {
//         use std::io::Read;

//         let mut file = std::fs::File::open(path).unwrap();
//         let mut contents: Vec<u8> = Vec::new();
//         file.read_to_end(&mut contents).unwrap();
//         contents
// }

impl HttpSignature for reqwest::RequestBuilder {
    fn http_sign_outgoing(self) -> Result<reqwest::RequestBuilder, Error> {
        // try and remove clone here
        let req = self.try_clone().unwrap().build().unwrap();
        let config = Config::default()
            .set_expiration(Duration::seconds(10))
            .dont_use_created_field();
        let server_key_id = SERVER.key_id.clone();
        let mut bt = std::collections::BTreeMap::new();
        for (k, v) in req.headers().iter() {
            bt.insert(k.as_str().to_owned(), v.to_str().unwrap().to_owned());
        }
        let path_and_query = if let Some(query) = req.url().query() {
            format!("{}?{}", req.url().path(), query)
        } else {
            req.url().path().to_string()
        };
        let unsigned = config
            .begin_sign(req.method().as_str(), &path_and_query, bt)
            .unwrap();
        let sig_header = unsigned
            .sign(server_key_id, |signing_string| {
                let private_key = read_file(Path::new(&env::var("SIGNATURE_PRIVKEY").unwrap()));
                let key_pair =
                    ring::signature::RsaKeyPair::from_pkcs8(&private_key.unwrap()).unwrap();
                let rng = ring::rand::SystemRandom::new();
                let mut signature = vec![0; key_pair.public_modulus_len()];
                key_pair
                    .sign(
                        &ring::signature::RSA_PKCS1_SHA256,
                        &rng,
                        signing_string.as_bytes(),
                        &mut signature,
                    )
                    .unwrap();
                // let digest = digest::digest(&digest::SHA256, &signing_string.as_bytes());
                let hexencode = base64::encode(&signature);
                Ok(hexencode) as Result<_, Error>
            })?
            .signature_header();
        // this SHOULD be OK
        // host and date?
        let result = self.header("Signature", sig_header);
        println!("{:?}", &result);
        Ok(result)
    }
}

fn sign_and_verify_rsa(
    private_key_path: &std::path::Path,
    public_key_path: &std::path::Path,
) -> Result<(), MyError> {
    use ring::{rand, signature};
    // Create an `RsaKeyPair` from the DER-encoded bytes. This example uses
    // a 2048-bit key, but larger keys are also supported.
    let private_key_der = read_file(private_key_path)?;
    let key_pair =
        signature::RsaKeyPair::from_pkcs8(&private_key_der).map_err(|_| MyError::BadPrivateKey)?;

    // Sign the message "hello, world", using PKCS#1 v1.5 padding and the
    // SHA256 digest algorithm.
    const MESSAGE: &'static [u8] = b"hello, world";
    let rng = rand::SystemRandom::new();
    let mut signature = vec![0; key_pair.public_modulus_len()];
    key_pair
        .sign(&signature::RSA_PKCS1_SHA256, &rng, MESSAGE, &mut signature)
        .map_err(|_| MyError::OOM)?;

    // Verify the signature.
    let public_key = signature::UnparsedPublicKey::new(
        &signature::RSA_PKCS1_2048_8192_SHA256,
        read_file(public_key_path)?,
    );
    public_key
        .verify(MESSAGE, &signature)
        .map_err(|_| MyError::BadSignature)
}

#[derive(Debug)]
enum MyError {
    IO(std::io::Error),
    BadPrivateKey,
    OOM,
    BadSignature,
}

fn read_file(path: &std::path::Path) -> Result<Vec<u8>, MyError> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|e| MyError::IO(e))?;
    let mut contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut contents)
        .map_err(|e| MyError::IO(e))?;
    Ok(contents)
}

use warp::http;

pub async fn verify_ap_message(
    method: &str,
    path_and_query: &str,
    headers: BTreeMap<String, String>,
) -> Result<(), Error> {
    // TODO -- case insensitivity?
    // mastodon doesnt use created filed
    let config = Config::default()
        .set_expiration(Duration::seconds(3600))
        .dont_use_created_field();
    let unverified = config.begin_verify(method, path_and_query, headers)?;
    let actor: Actor = get_remote_actor(unverified.key_id()).await?;
    let res = unverified.verify(|signature, signing_string| {
        let public_key: &[u8] = actor.public_key.public_key_pem.as_bytes();
        let r = Rsa::public_key_from_pem(public_key).unwrap();
        let public_key = r.public_key_to_der_pkcs1().unwrap();
        let key = UnparsedPublicKey::new(&ring::signature::RSA_PKCS1_2048_8192_SHA256, &public_key);
        let hexdecode = base64::decode(signature).unwrap();
        key.verify(signing_string.as_bytes(), &hexdecode).unwrap();
        true
    });
    Ok(())
}
