// @generated automatically by Diesel CLI.

diesel::table! {
    project (id) {
        id -> Nullable<Integer>,
        name -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    request (id) {
        id -> Nullable<Integer>,
        name -> Text,
        description -> Nullable<Text>,
        url -> Text,
        endpoint -> Text,
        project_id -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    variable (id) {
        id -> Nullable<Integer>,
        name -> Text,
        value -> Nullable<Text>,
        project_id -> Integer,
        created_at -> Timestamp,
    }
}

diesel::joinable!(request -> project (project_id));
diesel::joinable!(variable -> project (project_id));

diesel::allow_tables_to_appear_in_same_query!(project, request, variable,);
