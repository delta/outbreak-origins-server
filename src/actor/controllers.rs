use crate::actor::events::types::{
    ControlMeasure, ControlMeasureParams, Event, EventParams, SimulatorResponse, Start, WSResponse,
};
use diesel::PgConnection;

use crate::actor::utils::serialize_state;
use crate::auth::extractors;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use virus_simulator::Simulator;

const POPULATION: f64 = 10000000.0;
const TOTAL_DAYS: f64 = 700.0;

pub trait RequestType {
    fn handle(payload: String, user: &extractors::Authenticated, conn: &PgConnection)
        -> WSResponse;
}

impl RequestType for Start {
    fn handle(
        _payload: String,
        _user: &extractors::Authenticated,
        _conn: &PgConnection,
    ) -> WSResponse {
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

impl RequestType for ControlMeasure {
    fn handle(
        payload: String,
        _user: &extractors::Authenticated,
        _conn: &PgConnection,
    ) -> WSResponse {
        let control_measure = serde_json::from_str::<ControlMeasure>(&payload);

        let res = match control_measure {
            Err(_) => WSResponse::Error("Couldn't parse request".to_string()),

            Ok(control_measure) => {
                let file = format!("src/game/levels/{}/control.json", control_measure.level);
                let path = Path::new(&file);
                let contents =
                    fs::read_to_string(&path).expect("Something  went wrong reading the file");
                let control_measure_data =
                    serde_json::from_str::<HashMap<String, ControlMeasureParams>>(&contents)
                        .unwrap();

                match control_measure_data.get(&control_measure.name) {
                    Some(data) => match data.levels.get(&control_measure.level.to_string()) {
                        Some(level) => {
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

                            WSResponse::Control(SimulatorResponse {
                                payload,
                                ideal_reproduction_number: changed_params[0],
                                compliance_factor: changed_params[1],
                                recovery_rate: changed_params[2],
                                infection_rate: changed_params[3],
                            })
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

impl RequestType for Event {
    fn handle(
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
