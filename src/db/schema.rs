table! {
    notes (id) {
        id -> Integer,
        user_id -> Integer,
        in_reply_to -> Nullable<Integer>,
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
        admin -> Bool,
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
    notifications (id) {
        id -> Integer,
        notification_html -> Text,
        server_message -> Bool,
        created_time -> Timestamp,
    }
}

table! {
    notification_viewers (notification_id, user_id) {
        notification_id -> Integer,
        user_id -> Integer,
        viewed -> Bool,
    }
}

joinable!(sessions -> users (user_id));
joinable!(notes -> users (user_id));
joinable!(notification_viewers -> notifications (notification_id));

allow_tables_to_appear_in_same_query!(sessions, users);
allow_tables_to_appear_in_same_query!(notifications, notification_viewers);
allow_tables_to_appear_in_same_query!(notes, users);
