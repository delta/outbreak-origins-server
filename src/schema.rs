table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Nullable<Text>,
        curlevel -> Int4,
        email -> Text,
    }
}
