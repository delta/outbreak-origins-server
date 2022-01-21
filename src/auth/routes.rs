use crate::auth::{controllers, response};
use crate::db::models;
use crate::db::types::PgPool;
use actix_identity::Identity;
use actix_web::{get, post, web, Error, HttpResponse};

#[post("/user/register")]
async fn register_user(
    pool: web::Data<PgPool>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, Error> {
    println!("here");
    web::block(move || {
        let conn = pool.get()?;
        controllers::insert_new_user(
            &form.firstname,
            &form.lastname,
            &form.password,
            &form.email,
            &conn,
        )
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
    pool: web::Data<PgPool>,
    form: web::Json<models::NewUser>,
    id: Identity,
) -> Result<HttpResponse, Error> {
    let (is_verified, token, status) = web::block(move || {
        let conn = pool.get()?;
        controllers::verify_user_by_email(&form.email, &form.password, &conn)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    println!("Token: {}", token);
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
    cfg.service(
        web::scope("/auth")
            .service(register_user)
            .service(logout_user)
            .service(login_user),
    );
}
