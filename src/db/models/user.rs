use serde::{Deserialize, Serialize};

use crate::db::schema::users;

#[derive(Identifiable, Debug, Clone, Serialize, Deserialize, Queryable)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct LeaderboardEntry {
    pub firstname: String,
    pub lastname: String,
    pub score: i32,
    pub money: i32,
}
