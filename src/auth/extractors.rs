use crate::auth::error;
use crate::db::models;
use actix_web::FromRequest;
use futures::future::{ready, Ready};

pub type AuthenticationInfo = Option<models::Identity>;

#[derive(Debug)]
pub struct Authenticated(pub AuthenticationInfo);

impl FromRequest for Authenticated {
    type Config = ();
    type Error = error::AuthError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<AuthenticationInfo>().cloned();
        let result = match value {
            Some(v) => Ok(Authenticated(v)),
            None => Err(error::AuthError::NotAuthenticated),
        };
        ready(result)
    }
}

impl std::ops::Deref for Authenticated {
    type Target = AuthenticationInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
