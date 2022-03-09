use diesel::sql_types::Text;
use diesel::sql_types::{BigInt, Integer};
use serde::{Deserialize, Serialize};

use crate::db::schema::users;

#[derive(
    Identifiable, Debug, Clone, Serialize, Deserialize, QueryableByName, Queryable, AsChangeset,
)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub password: Option<String>,
    pub curlevel: i32,
    pub email: String,
    pub status: Option<i32>,
    pub firstname: String,
    pub lastname: String,
    pub score: i32,
    pub money: i32,
    pub is_email_verified: bool,
    pub is_active: bool,
    pub is_randomized: bool,
    pub retryattemptsleft: i32,
    pub is_level_active: bool,
    pub curr_level_score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub firstname: String,
    pub lastname: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct TestUser {
    pub firstname: String,
    pub lastname: String,
    pub password: String,
    pub email: String,
    pub score: i32,
    pub money: i32,
    pub is_email_verified: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterUser {
    pub firstname: String,
    pub lastname: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginUser {
    pub password: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, QueryableByName)]
pub struct LeaderboardEntry {
    #[sql_type = "Text"]
    pub firstname: String,
    #[sql_type = "Text"]
    pub lastname: String,
    #[sql_type = "Integer"]
    pub score: i32,
    #[sql_type = "BigInt"]
    pub rank: i64,
}
