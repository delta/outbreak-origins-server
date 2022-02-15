use crate::auth::{controllers, extractors, response};
use crate::db::models;
use crate::db::types::PgPool;
use actix_identity::Identity;
use actix_web::{get, http::StatusCode, post, web, Error, HttpResponse};

#[post("/user/register")]
async fn register_user(
    pool: web::Data<PgPool>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, Error> {
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

#[get("/checkauth")]
async fn check_auth(user: extractors::Authenticated) -> Result<HttpResponse, Error> {
    let email = user.0.as_ref().map(|y| y.email.clone());
    Ok(HttpResponse::Ok()
        .status(if user.is_some() {
            StatusCode::OK
        } else {
            StatusCode::UNAUTHORIZED
        })
        .json(response::CheckAuthResult {
            status: user.is_some(),
            email,
        }))
}

#[post("/user/login")]
async fn login_user(
    pool: web::Data<PgPool>,
    form: web::Json<models::LoginUser>,
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
    let resp = HttpResponse::Ok()
        .status(if is_verified {
            StatusCode::OK
        } else {
            StatusCode::UNAUTHORIZED
        })
        .json(response::AuthResult {
            is_verified,
            status,
        });
    Ok(resp)
}

// #[post("/user/verify?<token>&<email>")]
// async fn verify_user(
//     pool: web::Data<PgPool>,
//     token: web::Path<String>,
// ) -> Result<HttpResponse, Error> {
//     let (is_verified, status) = web::block(move || {
//         let conn = pool.get()?;
//         controllers::verify_user_by_token(&token, &conn)
//     })
//     .await
//     .map_err(|e| {
//         eprintln!("{}", e);
//         HttpResponse::InternalServerError().finish()
//     })?;
//     let resp = HttpResponse::Ok()
//         .status(if is_verified {
//             StatusCode::OK
//         } else {
//             StatusCode::UNAUTHORIZED
//         })
//         .json(response::AuthResult {
//             is_verified,
//             status,
//         });
//     Ok(resp)
// }


pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(register_user)
            .service(logout_user)
            .service(login_user)
            .service(check_auth),
    );
}
