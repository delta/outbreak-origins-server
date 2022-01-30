use crate::db::types::{PgPool, PgPooledConnection};
use crate::db::utils::find_event_by_id;

use virus_simulator::Simulator;

use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::{web, HttpResponse};
pub use actix_web_actors::ws;
use actix_web_actors::ws::{Message, ProtocolError};

use std::fmt::Write;
use std::fs;
use std::time::{Duration, Instant};
#[path = "utils.rs"]
mod utils;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

const INITIAL_INFECTED: f64 = 0.0000001;
const INITIAL_EXPOSED: f64 = 4.0 * INITIAL_INFECTED;
const INITIAL_SUSCEPTIBLE: f64 = 1.0 - INITIAL_INFECTED - INITIAL_EXPOSED;
const INITIAL_REMOVED: f64 = 0.0;
const INITIAL_REPRODUCTION_NUMBER: f64 = 1.6;

const INITIAL_IDEAL_REPRODUCTION_NUMBER: f64 = 2.0;
const INITIAL_RECOVERY_RATE: f64 = 0.0555;
const INITIAL_INFECTION_RATE: f64 = 0.1923076923;
const INITIAL_SOCIAL_PARAMETER: f64 = 0.5;

pub struct Game {
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
                if text == "START" {
                    let mut contents = fs::read_to_string("/Users/shankarkrishnamoorthy/Desktop/Delta/outbreak-origins-server/src/init_data.json")
        .expect("Something went wrong reading the file");

                    //println!("{}", contents);
                    let data = serde_json::from_str::<utils::types::InitParams>(&contents).unwrap();

                    //println!("{}", data.num_sections);

                    // Instance of Simulator
                    let sim = Simulator::new(
                        &data.section_data[0].init_params.susceptible,
                        &data.section_data[0].init_params.exposed,
                        &data.section_data[0].init_params.infectious,
                        &data.section_data[0].init_params.removed,
                        &data.section_data[0].init_params.current_reproduction_number,
                        &data.section_data[0].init_params.ideal_reproduction_number,
                        &data.section_data[0].init_params.compliance_factor,
                        &data.section_data[0].init_params.recovery_rate,
                        &data.section_data[0].init_params.infection_rate,
                    );

                    let f = sim.simulate(0_f64, 2_f64);
                    // serilising the data
                    let mut output = String::new();
                    output.write_str("{").unwrap();
                    for (j, state) in f.iter().enumerate() {
                        let mut i = 1;
                        output.write_str("[").unwrap();
                        for val in state.iter() {
                            if i % 5 == 0 {
                                output.write_fmt(format_args!("{} , ", val)).unwrap();
                            } else {
                                output
                                    .write_fmt(format_args!(
                                        "{} , ",
                                        val * data.section_data[0].population
                                    ))
                                    .unwrap();
                            }
                            output.pop();
                            output.pop();
                            i = i + 1;
                        }
                        output.write_str("], ").unwrap();
                        output.pop();
                        output.pop();
                        output.write_str(",").unwrap();
                    }
                    output.write_str("}").unwrap();
                    //println!("{}", output);

                    ctx.text(output);
                }
            }
            _ => ctx.stop(),
        }
    }

    fn started(&mut self, _ctx: &mut Self::Context) {
        let susceptible = INITIAL_SUSCEPTIBLE;
        let exposed = INITIAL_EXPOSED;
        let infected = INITIAL_INFECTED;
        let removed = INITIAL_REMOVED;
        let reproduction_number = INITIAL_REPRODUCTION_NUMBER;

        let ideal_reproduction_number = INITIAL_IDEAL_REPRODUCTION_NUMBER;
        let infection_rate = INITIAL_INFECTION_RATE;
        let recovery_rate = INITIAL_RECOVERY_RATE;
        let social_parameter = INITIAL_SOCIAL_PARAMETER;

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

        let _res = Simulator::simulate(simulator, 0.0, 700.0);
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

pub fn pg_pool_handler(pool: &web::Data<PgPool>) -> Result<PgPooledConnection, HttpResponse> {
    (*pool)
        .get()
        .map_err(|e| HttpResponse::InternalServerError().json(e.to_string()))
}
