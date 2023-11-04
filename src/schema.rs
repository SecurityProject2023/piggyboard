// @generated automatically by Diesel CLI.

diesel::table! {
    articles (id) {
        id -> Integer,
        author_id -> Integer,
        title -> Text,
        content -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    comments (id) {
        id -> Nullable<Integer>,
        author_id -> Integer,
        content -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    dislike (id) {
        id -> Integer,
        user_id -> Integer,
        article_id -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    like (id) {
        id -> Integer,
        user_id -> Integer,
        article_id -> Integer,
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

diesel::joinable!(articles -> users (author_id));
diesel::joinable!(comments -> users (author_id));
diesel::joinable!(dislike -> articles (article_id));
diesel::joinable!(dislike -> users (user_id));
diesel::joinable!(like -> articles (article_id));
diesel::joinable!(like -> users (user_id));
diesel::joinable!(posts -> users (author_id));

diesel::allow_tables_to_appear_in_same_query!(
    articles,
    comments,
    dislike,
    like,
    posts,
    users,
);
