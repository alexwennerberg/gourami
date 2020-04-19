table! {
    notes (id) {
        id -> Integer,
        creator_id -> Integer,
        creator_username -> Varchar,
        parent_id -> Nullable<Integer>,
        content -> Text,
        created_time -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Varchar,
        email -> Varchar,
        bio -> Text,
        created_time -> Timestamp,
        password -> Varchar,
    }
}

table! {
    sessions (id) {
        id -> Integer,
        cookie -> Varchar,
        user_id -> Integer,
        created_time -> Timestamp,
    }
}

joinable!(sessions -> users (user_id));
joinable!(notes -> users (creator_id));

allow_tables_to_appear_in_same_query!(sessions, users);
allow_tables_to_appear_in_same_query!(notes, users);
