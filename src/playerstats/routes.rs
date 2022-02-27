use crate::auth::extractors::Authenticated;
use crate::db::types::PgPool;
use crate::playerstats::{controllers, response};
use actix_web::{get, web, Error, HttpResponse};

#[get("/score")]
pub async fn score(pool: web::Data<PgPool>, user: Authenticated) -> Result<HttpResponse, Error> {
    let email = user.0.as_ref().map(|y| y.email.clone());
    let useremail = email.unwrap();
    let score = web::block(move || {
        let conn = pool.get()?;
        controllers::get_score(&conn, useremail)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().json(response::ScoreResponse {
            status: String::from("Failed"),
            data: 0,
        })
    })?;
    Ok(HttpResponse::Ok().json(response::ScoreResponse {
        status: String::from("Success"),
        data: score,
    }))
}

#[get("/money")]
pub async fn money(pool: web::Data<PgPool>, user: Authenticated) -> Result<HttpResponse, Error> {
    let email = user.0.as_ref().map(|y| y.email.clone());
    let useremail = email.unwrap();
    let money = web::block(move || {
        let conn = pool.get()?;
        controllers::get_money(&conn, useremail)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().json(response::MoneyResponse {
            status: String::from("Failed"),
            data: 0,
        })
    })?;
    Ok(HttpResponse::Ok().json(response::MoneyResponse {
        status: String::from("Success"),
        data: money,
    }))
}

pub fn stats_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/user/api/").service(score).service(money));
}