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

allow_tables_to_appear_in_same_query!(human_friends, humans,);
