use crate::db::models;
use crate::db::types::DbError;
use diesel::prelude::*;

pub fn get_leaderboard(conn: &PgConnection) -> Result<Vec<models::LeaderboardEntry>, DbError> {
    use crate::db::schema::users::dsl::*;
    let users_result: QueryResult<Vec<models::User>> = users.load(conn);
    let _my_users = users_result.expect("Error loading Users");

    let leaderboard_data = users
        .select((firstname, lastname, score))
        .order(score.desc())
        .load::<models::LeaderboardEntry>(conn)
        .expect("Error loading leaderboard");

    Ok(leaderboard_data)
}
