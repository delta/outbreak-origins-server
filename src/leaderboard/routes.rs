use crate::auth::extractors::Authenticated;
use crate::db::types::PgPool;
use crate::leaderboard::controllers::get_leaderboard;
use crate::leaderboard::response::LeaderboardResponse;
use actix_web::{get, web, Error, HttpResponse};
use tracing::instrument;

#[get("/{pg_num}")]
#[instrument(skip(pool))]
pub async fn leaderboard(
    web::Path(pg_num): web::Path<u32>,
    pool: web::Data<PgPool>,
    user: Authenticated,
) -> Result<HttpResponse, Error> {
    let (leaderboard, curr_user) = web::block(move || {
        let conn = pool.get()?;
        get_leaderboard(&conn, pg_num, user)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    Ok(HttpResponse::Ok().json(LeaderboardResponse {
        status: String::from("Success"),
        data: leaderboard,
        user_rank: curr_user,
    }))
}

pub fn leaderboard_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/leaderboard").service(leaderboard));
}
