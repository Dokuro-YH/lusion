table! {
    human_friends (human_id, friend_id) {
        human_id -> Uuid,
        friend_id -> Uuid,
    }
}

table! {
    humans (id) {
        id -> Uuid,
        name -> Text,
    }
}

table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        password -> Text,
        nickname -> Text,
        avatar_url -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

allow_tables_to_appear_in_same_query!(human_friends, humans, users,);
