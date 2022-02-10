use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use diesel::pg::{types::sql_types::Jsonb, Pg};
use diesel::serialize::Output;
use diesel::types::{FromSql, ToSql};
use std::io::Write;

use crate::db::schema::users;

#[derive(AsExpression, FromSqlRow, Serialize, Deserialize, Clone, Debug)]
#[sql_type = "Jsonb"]
pub struct ControlMeasureLevelInfo(pub HashMap<String, i32>);

impl FromSql<Jsonb, Pg> for ControlMeasureLevelInfo {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
        Ok(serde_json::from_value(value)?)
    }
}

impl ToSql<Jsonb, Pg> for ControlMeasureLevelInfo {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
        let value = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
    }
}

#[derive(Identifiable, Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub password: Option<String>,
    pub curlevel: i32,
    pub email: String,
    pub firstname: String,
    pub lastname: String,
    pub score: i32,
    pub money: i32,
    pub is_email_verified: bool,
    pub control_measure_level_info: ControlMeasureLevelInfo,
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
