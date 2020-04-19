table! {
    notes (id) {
        id -> Integer,
        creator_id -> Integer,
        parent_id -> Nullable<Integer>,
        content -> Text,
        published -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Text,
        password -> Text,
        email -> Text,
    }
}

table! {
    sessions (id) {
        id -> Integer,
        cookie -> Text,
        user_id -> Integer,
    }
}

joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(sessions, users);
