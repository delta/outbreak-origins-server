use crate::db::types::PgPool;

use virus_simulator::Simulator;

use crate::actor::events::types::{ControlMeasure, SimulatorResponse, WSRequest, WSResponse};
use crate::actor::types::ControlMeasureParams;
use crate::actor::{types::InitParams, utils::serialize_state};
use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::web;
pub use actix_web_actors::ws;
use actix_web_actors::ws::{Message, ProtocolError};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
const POPULATION: f64 = 10000000.0;
const TOTAL_DAYS: f64 = 700.0;

pub struct Game {
    heartbeat: Instant,
    _pool: web::Data<PgPool>,
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
                    payload: "".to_string(),
                });

                let res = match request.kind.as_str() {
                    "Start" => {
                        let path = Path::new("src/init_data.json");
                        let contents = fs::read_to_string(&path)
                            .expect("Something went wrong reading the file");

                        let data = serde_json::from_str::<InitParams>(&contents).unwrap();

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

                        let f = sim.simulate(0_f64, TOTAL_DAYS);

                        // serilising the data
                        let payload = serialize_state(&f, data.section_data[0].population);
                        WSResponse::Start(SimulatorResponse {
                            payload,
                            ideal_reproduction_number: data.section_data[0]
                                .init_params
                                .ideal_reproduction_number,
                            compliance_factor: data.section_data[0].init_params.compliance_factor,
                            recovery_rate: data.section_data[0].init_params.recovery_rate,
                            infection_rate: data.section_data[0].init_params.infection_rate,
                        })
                    }
                    "Control" => {
                        let control_result =
                            serde_json::from_str::<ControlMeasure>(&request.payload);
                        let res = match control_result {
                            Err(_) => WSResponse::Error("Invalid request sent".to_string()),
                            Ok(control) => {
                                // TODO: Change to be based on user level
                                let path = Path::new("src/game/levels/1/control.json");
                                let contents = fs::read_to_string(&path)
                                    .expect("Something  went wrong reading the file");
                                let control_measure_params =
                                    serde_json::from_str::<HashMap<String, ControlMeasureParams>>(
                                        &contents,
                                    )
                                    .unwrap();

                                match control_measure_params.get(&control.name) {
                                    Some(params) => {
                                        let initial = [
                                            control.params.ideal_reproduction_number,
                                            control.params.compliance_factor,
                                            control.params.recovery_rate,
                                            control.params.infection_rate,
                                        ];
                                        println!("{:?}", initial);
                                        let changed_params = params.params_delta.iter().fold(
                                            initial,
                                            |mut acc, x| {
                                                match x.name.as_str() {
                                                    "Ideal Reproduction" => acc[0] += x.value,
                                                    "Compliance Factor" => acc[1] += x.value,
                                                    "Recovery Rate" => acc[2] += x.value,
                                                    "Infection Rate" => acc[3] += x.value,
                                                    _ => {}
                                                };
                                                acc
                                            },
                                        );
                                        let susceptible = control.params.susceptible / POPULATION;
                                        let exposed = control.params.exposed / POPULATION;
                                        let infectious = control.params.infectious / POPULATION;
                                        let removed = control.params.removed / POPULATION;
                                        let cur_date = control.cur_date.clone();
                                        let sim = Simulator::new(
                                            &susceptible,
                                            &exposed,
                                            &infectious,
                                            &removed,
                                            &control.params.current_reproduction_number,
                                            &changed_params[0],
                                            &changed_params[1],
                                            &changed_params[2],
                                            &changed_params[3],
                                        );
                                        let f = sim.simulate(0_f64, TOTAL_DAYS - cur_date);
                                        let payload = serialize_state(&f, POPULATION);
                                        WSResponse::Control(SimulatorResponse {
                                            payload,
                                            ideal_reproduction_number: changed_params[0],
                                            compliance_factor: changed_params[1],
                                            recovery_rate: changed_params[2],
                                            infection_rate: changed_params[3],
                                        })
                                    }
                                    None => WSResponse::Error("Invalid request sent".to_string()),
                                }
                            }
                        };
                        res
                    }
                    _ => WSResponse::Error("Invalid request sent".to_string()),
                };
                ctx.text(res.stringify())
            }
            _ => ctx.stop(),
        }
    }
}

impl Game {
    pub fn new(conn_pool: web::Data<PgPool>) -> Self {
        Self {
            heartbeat: Instant::now(),
            _pool: conn_pool,
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
