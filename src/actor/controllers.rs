use crate::actor::events::types::{SimulatorResponse, ControlMeasure, WSResponse, CurParams};
use crate::actor::types::{ParamsDelta, ControlMeasureParams};
use serde::de::DeserializeOwned;
use std::path::Path;
use virus_simulator::Simulator;
use std::fs;
use crate::actor::utils::serialize_state;
use std::collections::HashMap;

const POPULATION: f64 = 10000000.0;
const TOTAL_DAYS: f64 = 700.0;

pub trait RequestType {
    fn type_name() -> String;
    fn name(&self) -> String;
    fn params(&self) -> CurParams;
    fn level(&self) -> i32;
    fn cur_date(&self) -> f64;
}

impl RequestType for ControlMeasure {
    fn type_name() -> String {
        "control".to_string()
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn params(&self) -> CurParams {
        self.params.clone()
    }
    fn level(&self) -> i32 {
        self.level
    }
    fn cur_date(&self) -> f64 {
        self.cur_date
    }
}

pub trait EventResponseType {
    fn params_delta(&self) -> Vec<ParamsDelta>;
}

impl EventResponseType for ControlMeasureParams {
    fn params_delta(&self) -> Vec<ParamsDelta> {
        self.params_delta.clone()
    }
}

pub fn handle_request<T: RequestType + DeserializeOwned, E: EventResponseType + DeserializeOwned>(
    payload: String,
) -> WSResponse {
    let ws_input = serde_json::from_str::<T>(&payload);
    let res = match ws_input {
        Err(_) => WSResponse::Error("Invalid request sent(Parsing)".to_string()),
        Ok(event) => {

            // TODO: Change to be based on user level
            let file = format!("src/game/levels/{}/{}.json", event.level(), T::type_name());
            let path = Path::new(&file);
            let contents = fs::read_to_string(&path)
                .expect("Something  went wrong reading the file");
            let control_measure_params =
                serde_json::from_str::<HashMap<String, ControlMeasureParams>>(
                    &contents,
                )
                .unwrap();

            let sent_params = event.params();
            match control_measure_params.get(&event.name()) {
                Some(params) => {
                    let initial = [
                        sent_params.ideal_reproduction_number,
                        sent_params.compliance_factor,
                        sent_params.recovery_rate,
                        sent_params.infection_rate,
                    ];
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
                    let susceptible = sent_params.susceptible / POPULATION;
                    let exposed = sent_params.exposed / POPULATION;
                    let infectious = sent_params.infectious / POPULATION;
                    let removed = sent_params.removed / POPULATION;
                    let cur_date = event.cur_date();
                    let sim = Simulator::new(
                        &susceptible,
                        &exposed,
                        &infectious,
                        &removed,
                        &sent_params.current_reproduction_number,
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
