use crate::actor::events::types::SimulatorParams;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;

use diesel::pg::{types::sql_types::Jsonb, Pg};
use diesel::serialize::Output;
use diesel::types::{FromSql, ToSql};

use crate::db::schema::{regions, regions_status, status};

#[derive(AsExpression, FromSqlRow, Serialize, Deserialize, Clone, Debug)]
#[sql_type = "Jsonb"]
pub struct ActiveControlMeasures(pub HashMap<String, i32>);

impl FromSql<Jsonb, Pg> for ActiveControlMeasures {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
        Ok(serde_json::from_value(value)?)
    }
}

impl ToSql<Jsonb, Pg> for ActiveControlMeasures {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
        let value = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
    }
}

impl FromSql<Jsonb, Pg> for SimulatorParams {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
        Ok(serde_json::from_value(value)?)
    }
}

impl ToSql<Jsonb, Pg> for SimulatorParams {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
        let value = serde_json::to_value(self)?;
        <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
    }
}

#[derive(Identifiable, Debug, Clone, Serialize, Deserialize, Queryable)]
#[table_name = "status"]
pub struct Status {
    pub id: i32,
    pub current_event: String,
    pub postponed: i32,
}

#[derive(Identifiable, Debug, Clone, Serialize, Deserialize, Queryable)]
#[table_name = "regions_status"]
pub struct RegionsStatus {
    pub id: i32,
    pub status_id: i32,
    pub region_id: i32,
}

#[derive(Identifiable, Debug, Clone, Serialize, Deserialize, Queryable)]
#[table_name = "regions"]
pub struct Regions {
    pub id: i32,
    pub region_id: i32,
    pub simulation_params: SimulatorParams,
    pub active_control_measures: ActiveControlMeasures,
}
