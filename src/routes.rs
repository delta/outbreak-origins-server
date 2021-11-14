// extern crate time;
use actix_web::{
    get,
    http::{Cookie, StatusCode},
    post, web, Error, HttpResponse,
};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
// use chrono::Duration;
use time::Duration;
// use time::duration::Duration;

use crate::actions;
use crate::models;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[get("/user")]
async fn get_user(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let user = web::block(move || {
        let conn = pool.get()?;
        actions::find_user_by_uid(&conn)
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

#[post("/user/register")]
async fn register_user(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, Error> {
    println!("here");
    web::block(move || {
        let conn = pool.get()?;
        actions::insert_new_user(&form.username, &form.password, &conn)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().json(models::AuthResult {
            is_verified: false,
            status: String::from("Couldn't create"),
        })
    })?;
    Ok(HttpResponse::Ok().json(models::AuthResult {
        is_verified: true,
        status: String::from("Created successfully"),
    }))
}

#[post("/user/login")]
async fn login_user(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, Error> {
    let (is_verified, token, status) = web::block(move || {
        let conn = pool.get()?;
        actions::verify_user_by_username(&form.username, &form.password, &conn)
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
    let mut resp = HttpResponse::Ok().json(models::AuthResult {
        is_verified: true,
        status: String::from("Successfully done"),
    });
    resp.add_cookie(&auth_cookie)?;
    Ok(resp)
}
