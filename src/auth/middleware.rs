use crate::db::models;
use actix_web::HttpMessage;
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
        let (user, iden) = match identity {
            None => (None, None),
            Some(iden) => (
                if let Ok(claim) = serde_json::from_str(&iden) {
                    Some(claim)
                } else {
                    None
                },
                Some(iden),
            ),
        };
        req.extensions_mut()
            .insert::<Option<models::Identity>>(user);
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            cookie_policy().to_response(iden, true, &mut res).await?;
            Ok(res)
        })
    }
}

pub fn cookie_policy() -> CookieIdentityPolicy {
    let expiry = std::env::var("EXPIRY")
        .expect("EXPIRY")
        .parse::<i64>()
        .expect("Needed a number for expiry");
    // let app_url = std::env::var("APP_URL").expect("APP_URL must be present");
    let cookie_key = std::env::var("COOKIE_KEY").expect("COOKIE_KEY");
    CookieIdentityPolicy::new(cookie_key.as_ref()) // <- construct cookie policy
        .domain("localhost")
        .name("OutBreakAuth")
        .path("/")
        .max_age(expiry * 60 * 60)
}
