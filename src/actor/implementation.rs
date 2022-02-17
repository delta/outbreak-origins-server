use crate::db::types::PgPool;

use virus_simulator::Simulator;

use crate::actor::events::types::{StartResponse, WSRequest, WSResponse};
use crate::actor::{types::InitParams, utils::serialize_state};
use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web::web;
pub use actix_web_actors::ws;
use actix_web_actors::ws::{Message, ProtocolError};
use std::fs;
use std::path::Path;

use std::time::{Duration, Instant};

use dotenv::dotenv;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sendgrid::{Destination, Mail, SGClient};
use std::env;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

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
                if text == WSRequest::Start.to_string() {
                    let path = Path::new("src/init_data.json");
                    let contents =
                        fs::read_to_string(&path).expect("Something went wrong reading the file");

                    let data = serde_json::from_str::<InitParams>(&contents).unwrap();

                    for n in data.section_data.iter() {
                        // Instance of Simulator
                        let sim = Simulator::new(
                            &n.init_params.susceptible,
                            &n.init_params.exposed,
                            &n.init_params.infectious,
                            &n.init_params.removed,
                            &n.init_params.current_reproduction_number,
                            &n.init_params.ideal_reproduction_number,
                            &n.init_params.compliance_factor,
                            &n.init_params.recovery_rate,
                            &n.init_params.infection_rate,
                        );

                        let f = sim.simulate(0_f64, 2_f64);

                        // serilising the data
                        let payload = serialize_state(&f, n.population);
                        ctx.text(WSResponse::Start(StartResponse { payload }).stringify());

                        dotenv().expect("Can't load environment variables");

                        let api_key =
                            env::var("SENDGRID_API_KEY").expect("SENDGRID_API_KEY must be set");

                        let rand_string: String = thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(30)
                            .map(char::from)
                            .collect();

                        println!("{}", rand_string);

                        let mail: Mail = Mail::new()
                            .add_to(Destination {
                                address: "mukundh.srivathsan.nitt@gmail.com",
                                name: "Mukundh",
                            })
                            .add_from("mukundhsrivathsan@gmail.com")
                            .add_subject("Hello World!")
                            .add_html("<h1>Hello World!</h1>");

                        let sgc = SGClient::new(api_key);

                        SGClient::send(&sgc, mail).expect("Failed to send email");
                    }
                }
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
