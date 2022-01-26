use crate::leaderboard::controllers::get_leaderboard;
use crate::leaderboard::response::LeaderboardResponse;
use crate::PgPool;
use actix_web::{get, web, Error, HttpResponse};

#[get("/leaderboard")]
pub async fn leaderboard(pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    let leaderboard = web::block(move || {
        let conn = pool.get()?;
        get_leaderboard(&conn)
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
