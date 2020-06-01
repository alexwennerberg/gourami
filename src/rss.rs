use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use rss::{Channel, ChannelBuilder, GuidBuilder, ItemBuilder};

use crate::{UserNote, PAGE_SIZE};
use crate::db::note;
use chrono::{DateTime, Utc};

pub fn build_feed(title: String, description: String, link: String, notes: &[UserNote]) -> Result<Channel, String> {
    let items = generate_rss_items(notes)?;
    let mut namespaces = HashMap::new();
    namespaces.insert(
        "dc".to_string(),
        "http://purl.org/dc/elements/1.1/".to_string(),
    );

    ChannelBuilder::default()
        .namespaces(namespaces)
        .title(title)
        .link(link)
        .description(description)
        .items(items)
        .build()
}

fn generate_rss_items(notes: &[UserNote]) -> Result<Vec<rss::Item>, String> {
    notes
        .iter()
        .take(PAGE_SIZE as usize)
        .map(|note| note.try_into())
        .collect()
}

impl TryFrom<&UserNote> for rss::Item {
    type Error = String;

    fn try_from(user_note: &UserNote) -> Result<Self, Self::Error> {
        let note = &user_note.note;
        let guid = GuidBuilder::default()
            .value(note::get_url(note.id))
            .permalink(true)
            .build()?;

        let dc_extension = rss::extension::dublincore::DublinCoreExtensionBuilder::default()
            .creators(vec![user_note.username.clone()])
            .build()?;

        let pub_date = DateTime::<Utc>::from_utc(note.created_time, Utc);
        ItemBuilder::default()
            .guid(Some(guid))
            .title(note.content.clone())
            .link(note::get_url(note.id))
            .description(note.content.clone())
            .pub_date(pub_date.to_rfc2822())
            .dublin_core_ext(dc_extension)
            .build()
    }
}

