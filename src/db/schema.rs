table! {
    events (id) {
        id -> Int4,
        name -> Varchar,
        description -> Varchar,
        compliance_factor -> Float8,
        infection_rate -> Float8,
        ideal_reproduction_number -> Float8,
    }
}

table! {
    regions (id) {
        id -> Int4,
        susceptible -> Float8,
        exposed -> Float8,
        infected -> Float8,
        removed -> Float8,
        reproduction_number -> Float8,
        control_measure_levels -> Nullable<Int4>,
        control_measure_isactive -> Bool,
    }
}

table! {
    status (id) {
        id -> Int4,
        level_number -> Int4,
        current_event -> Int4,
        postponed -> Int4,
        regions -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        password -> Nullable<Text>,
        curlevel -> Int4,
        email -> Text,
        firstname -> Text,
        lastname -> Text,
    }
}

joinable!(status -> events (current_event));
joinable!(status -> regions (regions));

allow_tables_to_appear_in_same_query!(events, regions, status,);
