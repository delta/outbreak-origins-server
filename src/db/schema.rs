table! {
    regions (id) {
        id -> Int4,
        region_id -> Int4,
        simulation_params -> Jsonb,
        active_control_measures -> Jsonb,
    }
}

table! {
    regions_status (id) {
        id -> Int4,
        status_id -> Int4,
        region_id -> Int4,
    }
}

table! {
    status (id) {
        id -> Int4,
        current_event -> Int4,
        postponed -> Int4,
        cur_date -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        password -> Nullable<Text>,
        curlevel -> Int4,
        email -> Text,
        status -> Nullable<Int4>,
        firstname -> Text,
        lastname -> Text,
        score -> Int4,
        money -> Int4,
        is_email_verified -> Bool,
    }
}

joinable!(regions_status -> regions (region_id));
joinable!(regions_status -> status (status_id));
joinable!(users -> status (status));

allow_tables_to_appear_in_same_query!(regions, regions_status, status, users,);
