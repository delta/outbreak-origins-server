use serde::{Deserialize, Serialize};

use crate::schema::users;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: Option<String>,
    pub curlevel: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub password: String,
}
