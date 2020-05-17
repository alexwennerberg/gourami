use super::schema::notes;
use crate::db::user::User;
use ammonia;
use chrono::Utc;
use maplit::hashset;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;

use crate::ap::SERVER;

/// This isn't queryable directly,
/// It only works when joined with the users table
///
#[derive(Queryable, Debug, QueryableByName, Associations, Clone, Deserialize, Serialize)]
#[belongs_to(User)]
#[table_name = "notes"]
pub struct Note {
    // rename RenderedNote
    pub id: i32,
    pub user_id: i32,
    pub in_reply_to: Option<i32>,
    pub content: String,
    pub created_time: chrono::NaiveDateTime,
    pub neighborhood: bool,
    pub is_remote: bool,
    pub remote_id: Option<String>,
}

pub fn get_url(note_id: i32) -> String {
    // TODO move domain url function
    format!("{}/note/{}", SERVER.global_id, note_id)
}

impl Note {
    // we make some modifications for outgoing notes
    pub fn get_content_for_outgoing(&self, username: &str) -> String {
        // remove first reply string
        // username not user id
        format!("{}:{}💬 {}", SERVER.domain, username, self.content)
    }


    pub fn relative_timestamp(&self) -> String {
        // Maybe use some fancy library here
        let diff = Utc::now()
            .naive_utc()
            .signed_duration_since(self.created_time);
        if diff.num_days() > 30 {
            return format!("{}", self.created_time.date());
        } else if diff.num_hours() > 24 {
            return format!("{}d", diff.num_days());
        } else if diff.num_minutes() > 60 {
            return format!("{}h", diff.num_hours());
        } else if diff.num_seconds() > 60 {
            return format!("{}m", diff.num_minutes());
        } else {
            return format!("{}s", diff.num_seconds());
        }
    }
}

/// Content in the DB is stored in plaintext (WILL BE)
/// We want to render it so that it is rendered in HTML
/// This basically just means escaping characters and adding
/// automatic URL parsing
///

/// Run on both write to db and read from db, for redundancy
/// Prevents malicious content from being rendered
/// See the mastodon page for inspiration: https://docs.joinmastodon.org/spec/activitypub/
/// This is currently very aggressive -- maybe we could loosen it a bit
/// We probably want to allow microformats and some accessibiltiy tags
/// Dont allow a so we cant have sneaky urls -- I'll do all the url parsing on my end.
pub fn remove_unacceptable_html(input_text: &str) -> String {
    let ok_tags = hashset!["br", "p", "span"];
    let html_clean = ammonia::Builder::default()
        .tags(ok_tags)
        .clean(input_text)
        .to_string();
    return html_clean;
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "notes"]
pub struct NoteInput {
    //pub id: i32, //unsigned?
    pub user_id: i32,
    pub content: String,
    pub in_reply_to: Option<i32>,
    pub neighborhood: bool,
}

#[derive(Insertable, Eq, PartialEq, Clone, Debug)]
#[table_name = "notes"]
pub struct RemoteNoteInput {
    pub user_id: i32,
    pub content: String,
    pub in_reply_to: Option<i32>,
    pub neighborhood: bool,
    pub is_remote: bool,
    pub remote_id: String,
}

/// We render the first >>[num] or note emoji as a reply, for threading.
pub fn get_reply(note_text: &str) -> Option<i32> {
    let re = Regex::new(r"\B(📝|>>)(\d+)").unwrap();
    match re.captures(note_text) {
        Some(t) => t.get(2).unwrap().as_str().parse().ok(),
        None => None,
    }
}

pub fn get_mentions(note_text: &str) -> Vec<String> {
    let re = Regex::new(r"\B(@)(\S+)").unwrap();
    re.captures_iter(note_text)
        .map(|c| String::from(&c[2]))
        .collect()
}

/// used for user-input
/// Parse links -- stolen from https://git.cypr.io/oz/autolink-rust/src/branch/master/src/lib.rs
/// TODO -- sanitize before write and then render links on read
pub fn parse_note_text(text: &str) -> String {
    // There shouldn't be any html tags in the db, but
    // Let's strip it out just in case
    let html_clean = remove_unacceptable_html(text);
    if text.len() == 0 {
        return String::new();
    }
    // this regex has to function after html parsing has happened. very weird.
    let re = Regex::new(
        r"(?ix)
        \b(([\w-]+://?|www[.])[^\s()<>]+(?:\([\w\d]+\)|([^[:punct:]\s]|/)))
    ",
    )
    .unwrap();
    let replace_str = "<a href=\"$0\">$0</a>";
    let urls_parsed = re
        .replace_all(&html_clean, &replace_str as &str)
        .to_string();
    let note_regex = Regex::new(r"\B(📝|&gt;&gt;)(\d+)").unwrap();
    let replace_str = "<a href=\"/note/$2\">$0</a>";
    let notes_parsed = note_regex
        .replace_all(&urls_parsed, &replace_str as &str)
        .to_string();
    let person_regex = Regex::new(r"\B(@)(\S+)").unwrap();
    let replace_str = "<a href=\"/user/$2\">$0</a>";
    let people_parsed = person_regex
        .replace_all(&notes_parsed, &replace_str as &str)
        .to_string();
    // TODO get mentions too
    return people_parsed;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert!(parse_note_text("") == "")
    }

    #[test]
    fn test_escape_html() {
        let example = "<script>haxxor</script>hi>";
        assert!(parse_note_text(example) == "hi&gt;");
    }

    #[test]
    fn test_string_without_urls() {
        let src = "<p>Some HTML</p>";
        assert!(parse_note_text(src) == src);
    }

    #[test]
    fn test_string_with_http_urls() {
        // TODO fix test
        let src = "Check this out: https://doc.rust-lang.org";
        let linked =
            "Check this out: <a href=\"https://doc.rust-lang.org\">https://doc.rust-lang.org</a>";
        assert!(parse_note_text(src) == linked)
    }

    #[test]
    fn test_string_with_mailto_urls() {
        let src = "Send spam to mailto://oz@cypr.io";
        assert!(
            parse_note_text(src)
                == "Send spam to <a href=\"mailto://oz@cypr.io\">mailto://oz@cypr.io</a>"
        )
    }

    #[test]
    fn test_user_replace() {
        let src = "@joe whats up @sally";
        let linked = "<a href=\"/user/joe\">@joe</a> whats up <a href=\"/user/sally\">@sally</a>";
        assert!(parse_note_text(src) == linked)
    }

    #[test]
    fn test_note_replace() {
        let src = "📝123 cool post >>456";
        let linked =
            "<a href=\"/note/123\">📝123</a> cool post <a href=\"/note/456\">&gt;&gt;456</a>";
        assert!(parse_note_text(src) == linked)
    }

    #[test]
    fn test_get_reply_simple() {
        let src = "📝123 cool post >>456";
        assert!(get_reply(src) == Some(123));
    }

    #[test]
    fn test_get_reply_none() {
        let src = "No reply in this tweet";
        assert!(get_reply(src) == None);
    }
}
