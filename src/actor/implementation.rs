use crate::db::types::PgPool;

use crate::actor::events::types::{
    ControlMeasure, Event, Save, Seed, Start, WSRequest, WSResponse,
};

use crate::db::types::DbError;
use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::web;
pub use actix_web_actors::ws;
use actix_web_actors::ws::{Message, ProtocolError};
use crate::actor::utils::decrypt_data;

use std::time::{Duration, Instant};

use crate::auth::extractors;

use tracing::{error, instrument};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Game {
    heartbeat: Instant,
    pool: web::Data<PgPool>,
    user: extractors::Authenticated,
}

impl Actor for Game {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        use crate::db::schema::users::dsl::*;
        use diesel::prelude::*;

        let conn = self.pool.get().expect("Couldn't get DB connection");

        let auth_user = self.user.0.as_ref().unwrap();
        diesel::update(users.filter(email.eq(auth_user.email.clone())))
            .set(is_active.eq(true))
            .execute(&*conn)
            .expect("Couldn't set user status");
        self.heartbeat(ctx);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        use crate::db::schema::users::dsl::*;
        use diesel::prelude::*;

        let conn = self.pool.get().expect("Couldn't get DB connection");

        let auth_user = self.user.0.as_ref().unwrap();
        diesel::update(users.filter(email.eq(auth_user.email.clone())))
            .set(is_active.eq(false))
            .execute(&*conn)
            .expect("Couldn't set user status");

        ctx.stop();
    }
}

#[instrument(skip(res))]
fn ws_response(res: Result<WSResponse, DbError>) -> WSResponse {
    match res {
        Ok(x) => x,
        Err(e) => {
            error!("{}", e);
            WSResponse::Error("Internal Server Error".to_string())
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Game {
    fn handle(&mut self, item: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(Message::Ping(item)) => {
                self.heartbeat = Instant::now();
                ctx.pong(&item);
            }
            Ok(Message::Pong(_)) => {
                self.heartbeat = Instant::now();
            }
            Ok(Message::Text(text)) => {
                let text = match decrypt_data(&text) {
                    Ok(x) => x,
                    Err(x) => {
                        error!("Decrypt data: {}", x);
                        "".to_string()
                    }
                };
                let request = serde_json::from_str::<WSRequest>(&text).unwrap_or(WSRequest {
                    kind: "".to_string(),
                    region: 0,
                    payload: "".to_string(),
                });

                let conn = self.pool.get().expect("Couldn't get DB connection");

                let res = match request.kind.as_str() {
                    "Seed" => ws_response(Seed::handle(&self.user, &conn)),
                    "Start" => ws_response(Start::handle(request.payload, &self.user, &conn)),
                    "Control" => {
                        ws_response(ControlMeasure::handle(request.payload, &self.user, &conn))
                    }
                    "Event" => ws_response(Event::handle(request.payload, &self.user, &conn)),
                    "Save" => ws_response(Save::handle(request.payload, &self.user, &conn)),
                    _ => WSResponse::Error("Invalid request sent".to_string()),
                };
                ctx.text(res.stringify())
            }
            _ => ctx.stop(),
        }
    }
}

impl Game {
    pub fn new(conn_pool: web::Data<PgPool>, user: extractors::Authenticated) -> Self {
        Self {
            heartbeat: Instant::now(),
            pool: conn_pool,
            user,
        }
    }

    pub fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // if heartbeat timed out
            if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}
