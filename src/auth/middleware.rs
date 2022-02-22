use crate::auth::utils::{create_jwt, get_info_token};
use crate::db::models::Claims;
use actix_web::HttpMessage;
use chrono::{Duration, TimeZone, Utc};
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_identity::{CookieIdentityPolicy, IdentityPolicy, RequestIdentity};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ok, Ready};
use futures::Future;

pub struct CheckAuth;

// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for CheckAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckAuthMiddleware { service })
    }
}

pub struct CheckAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service for CheckAuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        println!("You requested: {}", req.path());

        let identity = req.get_identity();
        let (id, email, exp, created_at, user) = match identity {
            None => (None, None, None, None, None),
            Some(iden) => {
                if let Ok(claim) = get_info_token(iden) {
                    (
                        Some(claim.claims.types.clone()),
                        Some(claim.claims.email.clone()),
                        Some(claim.claims.exp),
                        Some(claim.claims.created_at),
                        Some(claim.claims),
                    )
                } else {
                    (None, None, None, None, None)
                }
            }
        };
        req.extensions_mut().insert::<Option<Claims>>(user);
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            if let (Some(i), Some(u), Some(e), Some(cr)) = (id, email, exp, created_at) {
                let expiry = std::env::var("EXPIRY")
                    .expect("EXPIRY")
                    .parse::<i64>()
                    .expect("Needed a number");
                let max_age = std::env::var("MAX_AGE")
                    .expect("MAX_AGE")
                    .parse::<i64>()
                    .expect("Needed a number");
                let add = if expiry < 2 {
                    Duration::seconds(expiry * 30)
                } else {
                    Duration::hours(expiry / 2)
                };
                if Utc::now()
                    .checked_add_signed(add)
                    .expect("invalid timestamp")
                    .timestamp() as usize
                    >= e
                    && (if let Some(dur) = Utc
                        .timestamp(cr as i64, 0)
                        .checked_add_signed(Duration::hours(max_age))
                    {
                        dur > Utc::now()
                    } else {
                        false
                    })
                {
                    let identity = if let Ok(claims) = create_jwt(i, u, created_at) {
                        Some(claims)
                    } else {
                        None
                    };
                    cookie_policy()
                        .to_response(identity, true, &mut res)
                        .await?;
                }
            }

            println!("Response");
            Ok(res)
        })
    }
}

pub fn cookie_policy() -> CookieIdentityPolicy {
    let expiry = std::env::var("EXPIRY")
        .expect("EXPIRY")
        .parse::<i64>()
        .expect("Needed a number for expiry");
    let cookie_key = std::env::var("COOKIE_KEY").expect("COOKIE_KEY");
    CookieIdentityPolicy::new(cookie_key.as_ref()) // <- construct cookie policy
        .domain("localhost")
        .name("OutBreakAuth")
        .path("/")
        .max_age(expiry * 60 * 60)
}
