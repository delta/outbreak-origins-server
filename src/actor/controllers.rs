use crate::actor::events::types::{
    ControlMeasure, ControlMeasureParams, Event, EventParams, SimulatorResponse, Start, WSResponse,
};
use crate::db::models;
use diesel::prelude::*;
use diesel::PgConnection;

use crate::actor::utils::serialize_state;
use crate::auth::extractors;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use virus_simulator::Simulator;

const POPULATION: f64 = 10000000.0;
const TOTAL_DAYS: f64 = 700.0;

impl Start {
    pub fn handle(_user: &extractors::Authenticated, _conn: &PgConnection) -> WSResponse {
        let path = Path::new("src/init_data.json");
        let contents = fs::read_to_string(&path).expect("Something went wrong reading the file");

        let data = serde_json::from_str::<Start>(&contents).unwrap();

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

        let payload = serialize_state(&f, data.section_data[0].population);
        WSResponse::Start(SimulatorResponse {
            payload,
            ideal_reproduction_number: data.section_data[0].init_params.ideal_reproduction_number,
            compliance_factor: data.section_data[0].init_params.compliance_factor,
            recovery_rate: data.section_data[0].init_params.recovery_rate,
            infection_rate: data.section_data[0].init_params.infection_rate,
        })
    }
}

impl ControlMeasure {
    pub fn handle(
        payload: String,
        user: &extractors::Authenticated,
        conn: &PgConnection,
    ) -> WSResponse {
        use crate::db::schema::users::dsl::*;
        let control_measure = serde_json::from_str::<ControlMeasure>(&payload);
        let user = user.0.as_ref().unwrap();
        let user = users
            .filter(email.eq(user.email.clone()))
            .first::<models::User>(conn)
            .optional();
        let mut user = match user {
            Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
            Ok(x) => match x {
                None => return WSResponse::Error("User not found".to_string()),
                Some(y) => y,
            },
        };

        let res = match control_measure {
            Err(_) => WSResponse::Error("Couldn't parse request".to_string()),

            Ok(control_measure) => {
                let file = format!("src/game/levels/{}/control.json", user.curlevel);
                let path = Path::new(&file);
                let contents = match fs::read_to_string(&path) {
                    Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
                    Ok(val) => val,
                };

                let control_measure_data =
                    serde_json::from_str::<HashMap<String, ControlMeasureParams>>(&contents)
                        .unwrap();

                let control_level =
                    match user.control_measure_level_info.0.get(&control_measure.name) {
                        Some(x) => x,
                        None => {
                            user.control_measure_level_info
                                .0
                                .insert(control_measure.name.clone(), 1);
                            &1
                        }
                    };

                match control_measure_data.get(&control_measure.name) {
                    Some(data) => match data.levels.get(&(control_level).to_string()) {
                        Some(level) => {
                            if level.cost > user.money as u32 {
                                return WSResponse::Error("Not enough money".to_string());
                            }
                            let recvd_params = [
                                control_measure.params.ideal_reproduction_number,
                                control_measure.params.compliance_factor,
                                control_measure.params.recovery_rate,
                                control_measure.params.infection_rate,
                            ];

                            let changed_params: Vec<f64> = level
                                .params_delta
                                .iter()
                                .zip(recvd_params.iter())
                                .map(|(&a, &b)| a + b)
                                .collect();

                            let susceptible = control_measure.params.susceptible / POPULATION;
                            let exposed = control_measure.params.exposed / POPULATION;
                            let infectious = control_measure.params.infectious / POPULATION;
                            let removed = control_measure.params.removed / POPULATION;
                            let sim = Simulator::new(
                                &susceptible,
                                &exposed,
                                &infectious,
                                &removed,
                                &control_measure.params.current_reproduction_number,
                                &changed_params[0],
                                &changed_params[1],
                                &changed_params[2],
                                &changed_params[3],
                            );

                            let f = sim.simulate(0_f64, TOTAL_DAYS - control_measure.cur_date);
                            let payload = serialize_state(&f, POPULATION);

                            // Update user money after the function has ran successfully
                            match diesel::update(users.filter(email.eq(user.email)))
                                .set((
                                    money.eq(user.money - level.cost as i32),
                                    control_measure_level_data.eq(user.control_measure_level_info),
                                ))
                                .execute(conn)
                            {
                                Ok(_) => WSResponse::Control(SimulatorResponse {
                                    payload,
                                    ideal_reproduction_number: changed_params[0],
                                    compliance_factor: changed_params[1],
                                    recovery_rate: changed_params[2],
                                    infection_rate: changed_params[3],
                                }),
                                Err(_) => WSResponse::Error("Internal Server Error".to_string()),
                            }
                        }
                        None => WSResponse::Error("Invalid request sent".to_string()),
                    },
                    None => WSResponse::Error("Invalid request sent".to_string()),
                }
            }
        };
        res
    }
}

impl Event {
    pub fn handle(
        payload: String,
        _user: &extractors::Authenticated,
        _conn: &PgConnection,
    ) -> WSResponse {
        let event = serde_json::from_str::<Event>(&payload);

        let res = match event {
            Err(_) => WSResponse::Error("Couldn't parse request".to_string()),

            Ok(event) => {
                let file = format!("src/game/levels/{}/event.json", event.level);
                let path = Path::new(&file);
                let contents =
                    fs::read_to_string(&path).expect("Something  went wrong reading the file");
                let event_data =
                    serde_json::from_str::<HashMap<String, EventParams>>(&contents).unwrap();
                match event_data.get(&event.name) {
                    Some(data) => {
                        let recvd_params = [
                            event.params.ideal_reproduction_number,
                            event.params.compliance_factor,
                            event.params.recovery_rate,
                            event.params.infection_rate,
                        ];

                        let changed_params: Vec<f64> = data
                            .params_delta
                            .iter()
                            .zip(recvd_params.iter())
                            .map(|(&a, &b)| a + b)
                            .collect();

                        let susceptible = event.params.susceptible / POPULATION;
                        let exposed = event.params.exposed / POPULATION;
                        let infectious = event.params.infectious / POPULATION;
                        let removed = event.params.removed / POPULATION;
                        let sim = Simulator::new(
                            &susceptible,
                            &exposed,
                            &infectious,
                            &removed,
                            &event.params.current_reproduction_number,
                            &changed_params[0],
                            &changed_params[1],
                            &changed_params[2],
                            &changed_params[3],
                        );

                        let f = sim.simulate(0_f64, TOTAL_DAYS - event.cur_date);
                        let payload = serialize_state(&f, POPULATION);

                        WSResponse::Event(SimulatorResponse {
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
}
