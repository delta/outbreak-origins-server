use actix_cors::Cors;
use actix_web::{get, http::Cookie, post, web, Error, HttpResponse};
// use chrono::Duration;
use actix_web::http;
use time::Duration;

use crate::auth::response;
use crate::models;
use crate::auth::controllers;

// use time::duration::Duration;

#[get("/user")]
async fn get_user(pool: web::Data<models::DbPool>) -> Result<HttpResponse, Error> {
    let user = web::block(move || {
        let conn = pool.get()?;
        controllers::find_user_by_uid(&conn)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        let res = HttpResponse::NotFound().body("No user found");
        Ok(res)
    }
}

#[get("/dummy")]
async fn dummy() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

#[post("/user/register")]
async fn register_user(
    pool: web::Data<models::DbPool>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, Error> {
    println!("here");
    web::block(move || {
        let conn = pool.get()?;
        controllers::insert_new_user( &form.username, &form.password, &form.email,&conn)
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

#[post("/user/login")]
async fn login_user(
    pool: web::Data<models::DbPool>,
    form: web::Json<models::NewUser>,
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
    let expiry_date: i64 = std::env::var("EXPIRY").expect("EXPIRY").parse().unwrap();
    let mut auth_cookie = Cookie::new("OrientationAuth", token);
    auth_cookie.set_domain("localhost");
    auth_cookie.set_path("/");
    auth_cookie.set_http_only(true);
    // auth_cookie.set_max_age(Some(Duration::hours(expiry_date)));
    let mut resp = HttpResponse::Ok().json(response::AuthResult {
        is_verified: true,
        status: String::from("Successfully done"),
    });
    resp.add_cookie(&auth_cookie)?;
    Ok(resp)
}

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    let cors_config: Cors = Cors::default()
        .allowed_origin("http://localhost:8080")
        .allowed_methods(vec!["POST"])
        .allowed_headers(vec![
            http::header::CONTENT_TYPE,
            http::header::ACCESS_CONTROL_ALLOW_HEADERS,
            http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        ])
        // .expose_headers(vec![
        //     http::header::SET_COOKIE,
        //     http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        // ])
        .supports_credentials();
    cfg.service(
        web::scope("/auth")
            .service(get_user)
            .wrap(cors_config)
            .service(register_user)
            .service(login_user),
    );
}
