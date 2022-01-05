use crate::models::Claims;
use crate::utils::functions;
use actix_web::HttpMessage;
use chrono::{Duration, Utc};
use jsonwebtoken::TokenData;
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
        println!("Hi from start. You requested: {}", req.path());

        let identity = req.get_identity();
        let (id, username, exp, user) = if identity.is_none() {
            (None, None, None, None)
        } else {
            if let Ok(claim) = functions::get_info_token(identity.unwrap()) {
                (
                    Some(claim.claims.id.clone()),
                    Some(claim.claims.username.clone()),
                    Some(claim.claims.exp),
                    Some(claim),
                )
            } else {
                (None, None, None, None)
            }
        };
        req.extensions_mut()
            .insert::<Option<TokenData<Claims>>>(user);
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            if id.is_some() && username.is_some() && exp.is_some() {
                let expiry = std::env::var("EXPIRY")
                    .expect("EXPIRY")
                    .parse::<i64>()
                    .expect("Needed a number");
                if Utc::now()
                    .checked_add_signed(Duration::hours(expiry / 2))
                    .expect("invalid timestamp")
                    .timestamp() as usize
                    >= exp.unwrap()
                {
                    let identity =
                        if let Ok(claims) = functions::create_jwt(id.unwrap(), username.unwrap()) {
                            Some(claims)
                        } else {
                            None
                        };
                    cookie_policy()
                        .to_response(identity, true, &mut res)
                        .await?;
                }
            }

            println!("Hi from response");
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
