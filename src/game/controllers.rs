use crate::auth::extractors::Authenticated;
use crate::db::models;
use crate::db::models::status::ActiveControlMeasures;
use crate::db::schema::{regions, regions_status, users};
use crate::db::types::DbError;
use crate::game::response;
use crate::game::response::ActiveControlMeasuresResponse;

use diesel::prelude::*;
use diesel::PgConnection;

pub fn get_current_level(
    conn: &PgConnection,
    user_email: String,
) -> Result<(i32, bool, bool, i32), DbError> {
    use crate::db::schema::users::dsl::*;

    let user_result = &users
        .filter(email.eq(user_email))
        .load::<models::User>(conn)?[0];

    Ok((
        user_result.curlevel,
        user_result.is_randomized,
        user_result.is_level_active,
        user_result.retryattemptsleft,
    ))
}

pub fn change_level_type(
    conn: &PgConnection,
    user: Authenticated,
    level_type: bool,
) -> Result<bool, DbError> {
    let user_email = user.0.unwrap().email;
    let user_result = &users::table
        .filter(users::email.eq(user_email.clone()))
        .load::<models::User>(conn)?[0];
    let money = if level_type { 600 } else { 500 };
    if user_result.is_level_active {
        Ok(false)
    } else {
        diesel::update(users::table.filter(users::email.eq(user_email)))
            .set((
                users::is_level_active.eq(true),
                users::is_randomized.eq(level_type),
                users::money.eq(money),
            ))
            .execute(conn)?;
        Ok(true)
    }
}

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
    attempt_score: i32,
    user_money: f64,
) -> Result<String, DbError> {
    use crate::db::schema::users::dsl::*;
    let user_email = user.0.as_ref().map(|y| y.email.clone());

    let curr_score: i32 = users
        .filter(email.eq(user_email.as_ref().unwrap()))
        .load::<models::User>(conn)
        .unwrap()[0]
        .curr_level_score;

    let tries_left: i32 = users
        .filter(email.eq(user_email.as_ref().unwrap()))
        .load::<models::User>(conn)
        .unwrap()[0]
        .retryattemptsleft;

    let user_score = (curr_score * (3 - tries_left) + attempt_score) / (4 - tries_left);

    match diesel::update(users.filter(email.eq(user_email.unwrap())))
        .set((
            retryattemptsleft.eq(retryattemptsleft - 1),
            money.eq(user_money as i32),
            score.eq(score - curr_score + user_score),
            curr_level_score.eq(user_score),
            status.eq::<Option<i32>>(None),
            is_level_active.eq(false),
        ))
        .execute(conn)
    {
        Ok(_) => Ok("Updated User".to_string()),
        Err(e) => Err(DbError::from(e)),
    }
}
