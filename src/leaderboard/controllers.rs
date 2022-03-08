use crate::auth::extractors::Authenticated;
use crate::db::models;
use crate::db::types::DbError;
use crate::leaderboard::controllers::models::LeaderboardEntry;
use diesel::prelude::*;
use diesel::sql_query;

const PAGE_SIZE: u32 = 10;

pub fn get_leaderboard(
    conn: &PgConnection,
    pg_num: u32,
    user: Authenticated,
) -> Result<(Vec<models::LeaderboardEntry>, LeaderboardEntry), DbError> {
    let offset = PAGE_SIZE * (pg_num - 1);
    let user_email = user.0.as_ref().map(|y| y.email.clone()).unwrap();

    let leaderboard_data = sql_query(format!(
        "SELECT firstname, lastname, \
        score, rank() OVER (ORDER BY score DESC) AS rank FROM users \
        LIMIT {} OFFSET {};",
        PAGE_SIZE, offset
    ))
    .load(conn)
    .expect("Error loading leaderboard");

    let user_result: Vec<models::LeaderboardEntry> = sql_query(format!(
        "SELECT firstname, \
        lastname, score, rank from \
        (
            SELECT email, firstname, \
            lastname, score, rank() OVER (ORDER BY score DESC) AS rank \
            FROM users \
        ) as foo WHERE email = \'{}\';",
        user_email
    ))
    .load(conn)
    .expect("Error loading user");

    let curr_user = &user_result[0];

    Ok((leaderboard_data, curr_user.clone()))
}
