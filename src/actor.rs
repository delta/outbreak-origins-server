use crate::db::*;

use virus_simulator::Simulator;

use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::{web,Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use actix_web_actors::ws::{Message, ProtocolError};

use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

const initial_infected: f64 = 0.0000001;
const initial_exposed: f64 = 4.0 * initial_infected;
const initial_susceptible: f64 = 1.0 - initial_infected - initial_exposed;
const initial_removed: f64 = 0.0;
const initial_reproduction_number: f64 = 1.6;

const initial_ideal_reproduction_number: f64 = 2.0;
const initial_recovery_rate: f64 = 0.0555;
const initial_infection_rate: f64 = 0.1923076923;
const initial_social_parameter: f64 = 0.5;

struct Game {
    heartbeat: Instant,
    pool: web::Data<PgPool>,
}

impl Actor for Game {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Game {
    fn handle(&mut self, item: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        println!("WS: {:?}", item);

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
                let event = String::from("event1");
                let pool = &(self.pool);
                let pg_pool = pg_pool_handler(pool);
                if text == event {
                    let event_details =
                        find_event_by_id(&pg_pool.expect("Can't fetch event details"), 1)
                            .unwrap()
                            .unwrap();
                    println!("{}", event_details.name);
                }
            }
            _ => ctx.stop(),
        }
    }

    fn started(&mut self, ctx: &mut Self::Context) {
        let susceptible = initial_susceptible;
        let exposed = initial_exposed;
        let infected = initial_infected;
        let removed = initial_removed;
        let reproduction_number = initial_reproduction_number;

        let ideal_reproduction_number = initial_ideal_reproduction_number;
        let infection_rate = initial_infection_rate;
        let recovery_rate = initial_recovery_rate;
        let social_parameter = initial_social_parameter;

        let simulator = Simulator::new(
            &susceptible,
            &exposed,
            &infected,
            &removed,
            &reproduction_number,
            &ideal_reproduction_number,
            &social_parameter,
            &recovery_rate,
            &infection_rate,
        );

        let res = Simulator::simulate(simulator, 0.0, 700.0);
        for x in 0..700 {
            println!("{}", res[x]);
        }
    }
}

impl Game {
    pub fn new(conn_pool: web::Data<PgPool>) -> Self {
        Self {
            heartbeat: Instant::now(),
            pool: conn_pool,
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

pub fn pg_pool_handler(pool: &(web::Data<PgPool>)) -> Result<PgPooledConnection, HttpResponse> {
    (*pool)
        .get()
        .map_err(|e| HttpResponse::InternalServerError().json(e.to_string()))
}

pub async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    println! {"{:?}",r};
    let res = ws::start(Game::new(pool), &r, stream);
    println!("{:?}", res);
    res
}