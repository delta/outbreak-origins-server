use crate::auth::{controllers, extractors, response, utils};
use crate::db::models;
use crate::db::types::PgPool;
use actix_identity::Identity;
use actix_web::{get, http::StatusCode, post, web, Error, HttpResponse};

#[post("/user/register")]
async fn register_user(
    pool: web::Data<PgPool>,
    form: web::Json<models::RegisterUser>,
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
    println!("{:?}", status.clone());
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

#[get("/user/verify")]
async fn verify_user(
    pool: web::Data<PgPool>,
    params: web::Query<utils::UserVerify>,
) -> Result<HttpResponse, Error> {
    let (is_verified, status) = web::block(move || {
        let conn = pool.get()?;
        controllers::verify_user_by_token(params.email.as_str(), params.token.to_string(), &conn)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    let resp = HttpResponse::Ok()
        .status(if is_verified {
            StatusCode::OK
        } else {
            StatusCode::NOT_ACCEPTABLE
        })
        .json(response::AuthResult {
            is_verified,
            status,
        });
    Ok(resp)
}

#[get("/user/reset_password_email")]
async fn reset_password_email(
    form: web::Json<models::ResetPasswordEmail>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let email = form.email.clone();
    let name = web::block(move || {
        let conn = pool.get()?;
        controllers::get_user_email(&form.email, &conn)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    let resp =
        match name {
            Some(n) => {
                if let Ok(_) = utils::send_reset_password_mail(&email, &n) {
                    HttpResponse::Ok().json(response::ResetPasswordResult {
                        status: true,
                        message: "Password email sent".to_string(),
                    })
                } else {
                    HttpResponse::Ok().json(response::ResetPasswordResult {
                        status: false,
                        message: "Couldn't send email".to_string(),
                    })
                }
            }
            None => HttpResponse::Ok().status(StatusCode::BAD_REQUEST).json(
                response::ResetPasswordResult {
                    status: false,
                    message: "User not present".to_string(),
                },
            ),
        };
    Ok(resp)
}

#[post("/user/token_validate")]
async fn token_validate(
    user: extractors::Authenticated,
    form: web::Json<models::ResetToken>,
) -> Result<HttpResponse, Error> {
    if user.0.is_none() {
        return Ok(HttpResponse::Ok()
            .status(StatusCode::UNAUTHORIZED)
            .json(response::TokenValidateResult { status: false }));
    }
    Ok(HttpResponse::Ok().json(response::TokenValidateResult {
        status: utils::get_info_token(&form.token).is_ok(),
    }))
}

#[post("/user/change_password")]
async fn change_password(
    pool: web::Data<PgPool>,
    form: web::Json<models::ChangePassword>,
    user: extractors::Authenticated,
) -> Result<HttpResponse, Error> {
    if user.0.is_none() {
        return Ok(HttpResponse::Ok().status(StatusCode::UNAUTHORIZED).json(
            response::ChangePasswordResult {
                status: false,
                message: "User not authenticated".to_string(),
            },
        ));
    }
    if let Ok(token) = utils::get_info_token(&form.jwt) {
        if &token.claims.email != &user.0.unwrap().email {
            return Ok(HttpResponse::Ok().json(response::ChangePasswordResult {
                status: false,
                message: "Token mismatch".to_string(),
            }));
        }
        web::block(move || {
            let conn = pool.get()?;
            controllers::reset_password(&token.claims.email, &form.new_password, &conn)
        })
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
        Ok(HttpResponse::Ok().json(response::ChangePasswordResult {
            status: true,
            message: "Password successfully changed".to_string(),
        }))
    } else {
        Ok(HttpResponse::Ok().json(response::ChangePasswordResult {
            status: false,
            message: "Invalid Token".to_string(),
        }))
    }
}

pub fn auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(register_user)
            .service(logout_user)
            .service(login_user)
            .service(check_auth)
            .service(verify_user)
            .service(reset_password_email)
            .service(token_validate)
            .service(change_password),
    );
}
