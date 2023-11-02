// @generated automatically by Diesel CLI.

diesel::table! {
    comments (id) {
        id -> Nullable<Integer>,
        author_id -> Integer,
        content -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    posts (id) {
        id -> Integer,
        author_id -> Integer,
        title -> Text,
        content -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        profile_img -> Text,
        profile_thumbnail -> Text,
        profile_banner -> Text,
        first_name -> Text,
        last_name -> Nullable<Text>,
        gender -> Text,
        username -> Text,
        email -> Text,
        phone -> Nullable<Text>,
        lang -> Text,
        bio -> Text,
        password -> Text,
        created_at -> Timestamp,
    }
}

diesel::joinable!(comments -> users (author_id));
diesel::joinable!(posts -> users (author_id));

diesel::allow_tables_to_appear_in_same_query!(
    comments,
    posts,
    users,
);
