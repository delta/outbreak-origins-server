use actix_web::{http::StatusCode, web, Error, HttpRequest, HttpResponse};

use crate::actor::implementation;
use crate::auth;
use crate::db::types::PgPool;

pub async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    pool: web::Data<PgPool>,
    user: auth::extractors::Authenticated,
) -> Result<HttpResponse, Error> {
    println!("Here");
    // if user.is_none() {
    //     return Ok(HttpResponse::Ok().status(StatusCode::UNAUTHORIZED).finish());
    // }
    let res = implementation::ws::start(implementation::Game::new(pool, user), &r, stream);
    println!("{:?}", res);
    res
}
