use crate::auth::extractors::Authenticated;
use crate::db::models;
use crate::db::models::status::ActiveControlMeasures;
use crate::db::schema::{regions, regions_status, users};
use crate::db::types::DbError;
use crate::game::response;
use crate::game::response::ActiveControlMeasuresResponse;

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
        .select((regions::region_id, regions::active_control_measures))
        .load::<(i32, ActiveControlMeasures)>(conn)?;
    let acm = ActiveControlMeasuresResponse {
        active_control_measures: acm_tups
            .into_iter()
            .map(|(x, y)| (x.to_string(), y))
            .collect(),
    };
    Ok(acm)
}

pub fn update_user_at_level_end(
    conn: &PgConnection,
    user: Authenticated,
    user_score: i32,
) -> Result<String, DbError> {
    use crate::db::schema::users::dsl::*;
    let user_email = user.0.as_ref().map(|y| y.email.clone());

    match diesel::update(users.filter(email.eq(user_email.unwrap())))
        .set((curlevel.eq(curlevel + 1), money.eq(500), score.eq(score + user_score)))
        .execute(conn)
    {
        Ok(_) => Ok("Updated User".to_string()),
        Err(e) => Err(DbError::from(e)),
    }
}
