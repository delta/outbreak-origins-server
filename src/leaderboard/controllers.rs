use crate::db::models;
use crate::db::types::DbError;
use diesel::prelude::*;

const PAGE_SIZE: u32 = 20;

pub fn get_leaderboard(
    conn: &PgConnection,
    pg_num: u32,
) -> Result<Vec<models::LeaderboardEntry>, DbError> {
    use crate::db::schema::users::dsl::*;
    let users_result: QueryResult<Vec<models::User>> = users.load(conn);
    let _my_users = users_result.expect("Error loading Users");
    let offset = PAGE_SIZE * (pg_num - 1);

    let leaderboard_data = users
        .select((firstname, lastname, score, money))
        .order(score.desc())
        .offset(offset.into())
        .limit(PAGE_SIZE.into())
        .load::<models::LeaderboardEntry>(conn)
        .expect("Error loading leaderboard");

    Ok(leaderboard_data)
}
