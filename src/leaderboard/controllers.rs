use crate::auth::extractors::Authenticated;
use crate::db::models;
use crate::db::types::DbError;
use crate::leaderboard::controllers::models::LeaderboardEntry;
use diesel::prelude::*;

const PAGE_SIZE: u32 = 10;

pub fn get_leaderboard(
    conn: &PgConnection,
    pg_num: u32,
    user: Authenticated,
) -> Result<(Vec<models::LeaderboardEntry>, LeaderboardEntry), DbError> {
    use crate::db::schema::users::dsl::*;
    let users_result: QueryResult<Vec<models::User>> = users.load(conn);
    let _my_users = users_result.expect("Error loading Users");
    let offset = PAGE_SIZE * (pg_num - 1);
    let user_email = user.0.as_ref().map(|y| y.email.clone());

    let leaderboard_data = users
        .select((firstname, lastname, score, money))
        .order(score.desc())
        .offset(offset.into())
        .limit(PAGE_SIZE.into())
        .load::<models::LeaderboardEntry>(conn)
        .expect("Error loading leaderboard");

    let user_result = users
        .filter(email.eq(user_email.unwrap()))
        .select((firstname, lastname, score, money))
        .load::<models::LeaderboardEntry>(conn)
        .expect("Error loading user");

    let curr_user = user_result.first().unwrap();
    let cur_user = (*curr_user).clone();

    Ok((leaderboard_data, cur_user))
}
