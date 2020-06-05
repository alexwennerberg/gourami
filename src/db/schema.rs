table! {
    notes (id) {
        id -> Integer,
        user_id -> Integer,
        in_reply_to -> Nullable<Integer>,
        content -> Text,
        created_time -> Timestamp,
        neighborhood -> Bool,
        is_remote -> Bool,
        remote_id -> Nullable<Varchar>,
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
        email -> Nullable<Varchar>,
        website -> Varchar,
        bio -> Text,
        created_time -> Timestamp,
        password -> Nullable<Varchar>,
        admin -> Bool,
        show_email -> Bool,
        remote_user -> Bool,
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

table! {
    server_mutuals (id) {
        id -> Integer,
        actor_id -> Varchar,
        inbox_url -> Varchar,
        accepted -> Bool,
        followed_back -> Bool,
        outbox_url -> Nullable<Varchar>,
    }
}

joinable!(sessions -> users (user_id));
joinable!(notes -> users (user_id));

allow_tables_to_appear_in_same_query!(sessions, users);
allow_tables_to_appear_in_same_query!(notes, users);
