table! {
    status (id) {
        id -> Integer,
        creator_id -> Integer,
        parent_id -> Nullable<Integer>,
        content -> Text,
        published -> Timestamp,
    }
}
