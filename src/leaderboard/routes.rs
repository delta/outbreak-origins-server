use crate::leaderboard::controllers::get_leaderboard;
use crate::leaderboard::response::LeaderboardResponse;
use crate::PgPool;
use actix_web::{get, web, Error, HttpResponse};

#[get("/{pg_num}")]
pub async fn leaderboard(
    web::Path(pg_num): web::Path<u32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let leaderboard = web::block(move || {
        let conn = pool.get()?;
        get_leaderboard(&conn, pg_num)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().json(LeaderboardResponse {
            status: String::from("Failed"),
            data: vec![],
        })
    })?;
    Ok(HttpResponse::Ok().json(LeaderboardResponse {
        status: String::from("Success"),
        data: leaderboard,
    }))
}

pub fn leaderboard_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/leaderboard").service(leaderboard));
}
