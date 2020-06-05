table! {
    activities (id) {
        id -> Integer,
        json_text -> Nullable<Text>,
    }
}

table! {
    notes (id) {
        id -> Integer,
        user_id -> Nullable<Integer>,
        content -> Nullable<Text>,
        created_time -> Nullable<Timestamp>,
        in_reply_to -> Nullable<Binary>,
        neighborhood -> Nullable<Bool>,
        is_remote -> Nullable<Bool>,
        remote_url -> Nullable<Text>,
        remote_creator -> Nullable<Text>,
        remote_id -> Nullable<Text>,
    }
}

table! {
    registration_keys (value) {
        value -> Nullable<Text>,
    }
}

table! {
    server_mutuals (id) {
        id -> Nullable<Integer>,
        actor_id -> Nullable<Text>,
        inbox_url -> Nullable<Text>,
        accepted -> Nullable<Bool>,
        followed_back -> Nullable<Bool>,
        outbox_url -> Nullable<Text>,
    }
}

table! {
    sessions (id) {
        id -> Integer,
        cookie -> Text,
        user_id -> Integer,
        created_time -> Nullable<Timestamp>,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Nullable<Text>,
        email -> Nullable<Text>,
        bio -> Nullable<Text>,
        website -> Varchar,
        created_time -> Nullable<Timestamp>,
        password -> Nullable<Text>,
        admin -> Nullable<Bool>,
        show_email -> Nullable<Bool>,
        remote_user -> Nullable<Bool>,
    }
}

joinable!(notes -> users (user_id));
joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(
    activities,
    notes,
    registration_keys,
    server_mutuals,
    sessions,
    users,
);
