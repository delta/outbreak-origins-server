use actix_web::{http::StatusCode, web, Error, HttpRequest, HttpResponse};

use crate::actor::implementation;
use crate::auth;
use crate::db::models::User;
use crate::db::types::PgPool;
use diesel::prelude::*;

use tracing::{info, instrument};

#[instrument(skip(r, stream, pool))]
pub async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    pool: web::Data<PgPool>,
    user: auth::extractors::Authenticated,
) -> Result<HttpResponse, Error> {
    use crate::db::schema::users::dsl::*;

    let conn = pool.get().expect("Couldn't get DB connection");
    let auth_user = user.0.as_ref().unwrap();
    let auth_user = users
        .filter(email.eq(auth_user.email.clone()))
        .first::<User>(&*conn)
        .expect("User not found");

    if user.is_none() {
        return Ok(HttpResponse::Ok().status(StatusCode::UNAUTHORIZED).finish());
    }

    // Don't allow multiple simultaneous connections
    if auth_user.is_active {
        info!("Another session is active");
        return Ok(HttpResponse::Ok().status(StatusCode::FORBIDDEN).finish());
    }

    implementation::ws::start(implementation::Game::new(pool, user), &r, stream)
}
