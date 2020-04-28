use maplit::hashset;
use super::schema::notes;
use serde::{de::Error, Deserialize, Serialize, Deserializer}; 
use regex::Regex;
use ammonia;
use crate::db::user::User; // weird import

/// This isn't queryable directly,
/// It only works when joined with the users table
///
#[derive(Queryable, Associations, Clone, Deserialize, Serialize)]
#[belongs_to(User)]
pub struct Note { // rename RenderedNote
  pub id: i32,
  pub user_id: i32,
  pub in_reply_to: Option<i32>,
  // deserialize wiht
  pub content: String,
  pub created_time: String,
  pub neighborhood: bool,
}

/// Content in the DB is stored in plaintext (WILL BE)
/// We want to render it so that it is rendered in HTML
/// This basically just means escaping characters and adding 
/// automatic URL parsing
fn render_content<'de, D>(deserializer: D) -> Result<String, D::Error>
where D: Deserializer<'de> {
   let s: &str = Deserialize::deserialize(deserializer)?;
    return Ok(parse_note_text(s));
}

#[derive(Insertable, Clone)]
#[table_name = "notes"]
pub struct NoteInput {
  //pub id: i32, //unsigned?
  pub user_id: i32,
  pub content: String, // can we make this a slice?
  pub in_reply_to: Option<i32>,
  pub neighborhood: bool,
}

/// We render the first >>[num] or note emoji as a reply, for threading.
pub fn get_reply(note_text: &str) -> Option<i32> {
    let re = Regex::new(r"\B(📝|>>)(\d+)").unwrap();
    match re.captures(note_text) {
        Some(t) => t.get(2).unwrap().as_str().parse().ok(),
        None => None
   }
}

/// used for user-input
/// Parse links -- stolen from https://git.cypr.io/oz/autolink-rust/src/branch/master/src/lib.rs
/// TODO -- sanitize before write and then render links on read
pub fn parse_note_text(text: &str) -> String {
    // There shouldn't be any html tags in the db, but
    // Let's strip it out just in case
    let html_clean = ammonia::clean_text(text);
    if text.len() == 0 {
        return String::new();
    }
    // this regex has to function after html parsing has happened. very weird.
    let re = Regex::new(
        r"(?ix)
        \b(([\w-]+:&\#47;&\#47;?|www[.])[^\s()<>]+(?:\([\w\d]+\)|([^[:punct:]\s]|&\#47)))
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

    #[test]
    fn test_escape_html() {
        let example = "<script>haxxor</script>hi>";
        assert!(parse_note_text(example) == ammonia::clean_text(example));
    }

    #[test]
    fn test_string_without_urls() {
        let src = "<p>Some HTML</p>";
        assert!(parse_note_text(src) == ammonia::clean_text(src));
    }

    #[test]
    fn test_string_with_http_urls() {
        let src = "Check this out: https://doc.rust-lang.org/\n
               https://fr.wikipedia.org/wiki/Caf%C3%A9ine";
        let linked = "Check&#32;this&#32;out:&#32;<a href=\"https:&#47;&#47;doc.rust-lang.org&#47;&#10;&#10;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;https:&#47;&#47;fr.wikipedia.org&#47;wiki&#47;Caf%C3%A9ine\">https:&#47;&#47;doc.rust-lang.org&#47;&#10;&#10;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;&#32;https:&#47;&#47;fr.wikipedia.org&#47;wiki&#47;Caf%C3%A9ine</a>";
        assert!(parse_note_text(src) == linked)
    }

    #[test]
    fn test_string_with_mailto_urls() {
        let src = "Send spam to mailto://oz@cypr.io";
        assert!(
            parse_note_text(src)
                == "Send&#32;spam&#32;to&#32;<a href=\"mailto:&#47;&#47;oz@cypr.io\">mailto:&#47;&#47;oz@cypr.io</a>"
        )
    }

    #[test]
    fn test_user_replace() {
        let src = "@joe whats up @sally";
        let linked = "<a href=\"/user/joe\">@joe</a>&#32;whats&#32;up&#32;<a href=\"/user/sally\">@sally</a>";
        assert!(parse_note_text(src) == linked)
    }

    #[test]
    fn test_note_replace() {
        let src = "📝123 cool post >>456";
        let linked = "<a href=\"/note/123\">📝123</a>&#32;cool&#32;post&#32;<a href=\"/note/456\">&gt;&gt;456</a>";
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
