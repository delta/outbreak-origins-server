use crate::auth::extractors::Authenticated;
use crate::db::models;
use crate::db::models::status::ActiveControlMeasures;
use crate::db::schema::{regions, regions_status, users};
use crate::db::types::DbError;
use crate::game::response;
use crate::game::response::ActiveControlMeasuresResponse;
use crate::game::utils;
use diesel::prelude::*;
use diesel::PgConnection;

pub fn get_active_control_measures(
    conn: &PgConnection,
    user: Authenticated,
) -> Result<response::ActiveControlMeasuresResponse, DbError> {
    let user = user.0.as_ref().unwrap();
    let user = (users::table)
        .filter(users::email.eq(user.email.clone()))
        .first::<models::User>(conn)?;

    let user_status_id = user.status.ok_or("Game hasn't started")?;
    let acm_tups = (regions::table)
        .filter(
            regions::id.eq_any(
                regions_status::table
                    .filter(regions_status::status_id.eq(user_status_id))
                    .select(regions_status::region_id)
                    .load::<i32>(conn)?,
            ),
        )
        .select((regions::id, regions::active_control_measures))
        .load::<(i32, ActiveControlMeasures)>(conn)?;
    let acm = ActiveControlMeasuresResponse {
        num_control_measures: acm_tups.len(),
        active_control_measures: utils::get_acm_map_from_db_res(&acm_tups),
    };
    Ok(acm)
}
