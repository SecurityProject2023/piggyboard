// @generated automatically by Diesel CLI.

diesel::table! {
    article_acl (id) {
        id -> Integer,
        article_id -> Integer,
        pread -> Integer,
        pedit -> Integer,
        pdelete -> Integer,
        prate -> Integer,
        pread_rate -> Integer,
        prv -> Integer,
        pwd -> Integer,
        ped -> Integer,
        pview_history -> Integer,
        pread_comments -> Integer,
        pwrite_comments -> Integer,
        pedit_comments -> Integer,
        pdelete_comments -> Integer,
    }
}

diesel::table! {
    article_history (id) {
        id -> Integer,
        article_id -> Integer,
        title -> Text,
        content -> Text,
        created_at -> Timestamp,
        who -> Integer,
    }
}

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
    comment_history (id) {
        id -> Integer,
        comment_id -> Integer,
        content -> Text,
        created_at -> Timestamp,
        who -> Integer,
    }
}

diesel::table! {
    comments (id) {
        id -> Integer,
        author_id -> Integer,
        article_id -> Integer,
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
        verified -> Bool,
        acl -> Text,
        created_at -> Timestamp,
    }
}

diesel::joinable!(article_acl -> articles (article_id));
diesel::joinable!(article_history -> articles (article_id));
diesel::joinable!(article_history -> users (who));
diesel::joinable!(articles -> users (author_id));
diesel::joinable!(comment_history -> comments (comment_id));
diesel::joinable!(comment_history -> users (who));
diesel::joinable!(comments -> articles (article_id));
diesel::joinable!(comments -> users (author_id));
diesel::joinable!(dislike -> articles (article_id));
diesel::joinable!(dislike -> users (user_id));
diesel::joinable!(like -> articles (article_id));
diesel::joinable!(like -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    article_acl,
    article_history,
    articles,
    comment_history,
    comments,
    dislike,
    like,
    users,
);
