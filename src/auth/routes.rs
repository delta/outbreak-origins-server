use actix_cors::Cors;
use actix_identity::Identity;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::http;
use actix_web::{get, post, web, Error, HttpResponse};

use crate::auth::{controllers, response};
use crate::models;

#[post("/user/register")]
async fn register_user(
    pool: web::Data<models::DbPool>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, Error> {
    println!("here");
    web::block(move || {
        let conn = pool.get()?;
        controllers::insert_new_user(&form.username, &form.password, &form.email, &conn)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().json(response::AuthResult {
            is_verified: false,
            status: String::from("Couldn't create"),
        })
    })?;
    Ok(HttpResponse::Ok().json(response::AuthResult {
        is_verified: true,
        status: String::from("Created successfully"),
    }))
}

#[get("/user/logout")]
async fn logout_user(id: Identity) -> Result<HttpResponse, Error> {
    id.forget();
    let resp = HttpResponse::Ok().json(response::LogoutResult { status: true });
    Ok(resp)
}

#[post("/user/login")]
async fn login_user(
    pool: web::Data<models::DbPool>,
    form: web::Json<models::NewUser>,
    id: Identity,
) -> Result<HttpResponse, Error> {
    let (is_verified, token, status) = web::block(move || {
        let conn = pool.get()?;
        controllers::verify_user_by_username(&form.username, &form.password, &conn)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    println!("{}", token);
    if is_verified {
        id.remember(token)
    }
    let resp = HttpResponse::Ok().json(response::AuthResult {
        is_verified,
        status,
    });
    Ok(resp)
}

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    let cors_config: Cors = Cors::default()
        .allowed_origin("http://localhost:8001")
        .allowed_methods(vec!["POST"])
        .allowed_headers(vec![
            http::header::CONTENT_TYPE,
            http::header::ACCESS_CONTROL_ALLOW_HEADERS,
            http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        ])
        .supports_credentials();
    let expiry = std::env::var("EXPIRY")
        .expect("EXPIRY")
        .parse::<i64>()
        .expect("Needed a number for expiry");
    let cookie_key = std::env::var("COOKIE_KEY").expect("COOKIE_KEY");
    cfg.service(
        web::scope("/auth")
            .wrap(cors_config)
            .service(register_user)
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(cookie_key.as_ref()) // <- construct cookie policy
                    .domain("localhost")
                    .name("OutBreakAuth")
                    .path("/")
                    .max_age(expiry),
            ))
            .service(login_user)
            .service(logout_user),
    );
}
