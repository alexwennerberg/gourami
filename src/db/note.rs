use chrono;
use std::collections::HashSet;
use activitystreams::object::streams;
use diesel::sqlite::SqliteConnection;
use maplit::hashset;
use diesel::deserialize::{Queryable};
use super::schema::notes;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use regex::Regex;
use std::iter::FromIterator;
use ammonia;

// Statuses are note activitystream object

#[derive(Queryable, Clone, Deserialize, Serialize)]
pub struct Note {
  pub id: i32,
  pub creator_id: i32,
  pub creator_username: String,
  pub parent_id: Option<i32>,
  pub content: String,
  pub created_time: String,
}

#[derive(Insertable, Clone)]
#[table_name = "notes"]
pub struct NoteInput {
  //pub id: i32, //unsigned?
  pub creator_id: i32,
  pub creator_username: String,
  pub parent_id: Option<i32>,
  pub content: String, // can we make this a slice?
  // pub published: chrono::NaiveDateTime,
}

/// used when we get content from another server
/// Derived from the big elephant
/// https://github.com/tootsuite/mastodon/blob/master/app/lib/sanitize_config.rb
pub fn sanitize_remote_content(html_string: &str) -> String {
    let ok_tags = hashset!["p", "br", "span", "a"];
    let html_clean = ammonia::Builder::new()
        .tags(ok_tags)
        .clean(html_string)
        .to_string();
    // this is OK for now -- but we want to add microformats like mastodon does
    html_clean
}

/// used for user-inpul
/// Parse links -- stolen from https://git.cypr.io/oz/autolink-rust/src/branch/master/src/lib.rs
pub fn parse_note_text(text: &str) -> String {
    // dont hack me
    let html_clean = ammonia::clean_text(text);
    if text.len() == 0 {
        return String::new();
    }
    let re = Regex::new(
        r"(?ix)
        \b(([\w-]+:&\#48;&\#47;?|www[.])[^\s()<>]+(?:\([\w\d]+\)|([^[:punct:]\s]|/)))
    ",
    )
    .unwrap();
    let replace_str = "<a href=\"$0\">$0</a>";
    let urls_parsed = re.replace_all(&html_clean, &replace_str as &str).to_string();
	let note_regex = Regex::new(
		r"\B(📝|&gt;&gt;)(\d+)",
	).unwrap();
	let replace_str = "<a href=\"/note/$2\">$0</a>";
	let notes_parsed = note_regex.replace_all(&urls_parsed, &replace_str as &str).to_string();
    let person_regex = Regex::new(
		r"\B(@)(\w+)").unwrap();
	let replace_str = "<a href=\"/user/$2\">$0</a>";
	let people_parsed = person_regex.replace_all(&notes_parsed, &replace_str as &str).to_string();
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

    fn test_escape_html() {
        assert!(parse_note_text("<script>haxxor</script>hi>") == "hi")
    }

    #[test]
    fn test_string_without_urls() {
        let src = "<p>Some HTML</p>";
        assert!(parse_note_text(src) == "Some HTML")
    }

    #[test]
    fn test_string_with_http_urls() {
        let src = "Check this out: https://doc.rust-lang.org/\n
               https://fr.wikipedia.org/wiki/Caf%C3%A9ine";
        let linked = "Check this out: <a href=\"https://doc.rust-lang.org/\">https://doc.rust-lang.org/</a>\n
               <a href=\"https://fr.wikipedia.org/wiki/Caf%C3%A9ine\">https://fr.wikipedia.org/wiki/Caf%C3%A9ine</a>";
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
    fn test_string_with_trailing_chars() {
        let src = "I love https://cat-bounce.com!\n
            Have you seen https://en.wikipedia.org/wiki/Cat_(disambiguation)?";
        let linked = "I love <a href=\"https://cat-bounce.com\">https://cat-bounce.com</a>!\n
            Have you seen <a href=\"https://en.wikipedia.org/wiki/Cat_(disambiguation)\">https://en.wikipedia.org/wiki/Cat_(disambiguation)</a>?";
        assert!(parse_note_text(src) == linked)
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
        let linked = "<a href=\"/note/123\">📝123</a> cool post <a href=\"/note/456\">&gt;&gt;456</a>";
        assert!(parse_note_text(src) == linked)
    }
}
