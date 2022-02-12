use crate::db::types::PgPool;

use crate::actor::events::types::{ControlMeasure, Event, Seed, Start, WSRequest, WSResponse};

use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::web;
pub use actix_web_actors::ws;
use actix_web_actors::ws::{Message, ProtocolError};
use serde_json;

use std::time::{Duration, Instant};

use crate::auth::extractors;

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
        self.heartbeat(ctx);
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
                println!("{}", text);
                let request = serde_json::from_str::<WSRequest>(&text).unwrap_or(WSRequest {
                    kind: "".to_string(),
                    region: 0,
                    payload: "".to_string(),
                });

                let conn = self.pool.get().expect("Couldn't get DB connection");

                let res = match request.kind.as_str() {
                    "Seed" => Seed::handle(&self.user, &conn),
                    "Start" => Start::handle(request.payload, &self.user, &conn),
                    "Control" => ControlMeasure::handle(request.payload, &self.user, &conn),
                    "Event" => Event::handle(request.payload, &self.user, &conn),
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
            // check client heartbeats
            if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}
