table! {
    notes (id) {
        id -> Integer,
        user_id -> Integer,
        parent_id -> Nullable<Integer>,
        content -> Text,
        created_time -> Timestamp,
    }
}

table! {
    registration_keys (value) {
        value -> Varchar,
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
joinable!(notes -> users (user_id));

allow_tables_to_appear_in_same_query!(sessions, users);
allow_tables_to_appear_in_same_query!(notes, users);
