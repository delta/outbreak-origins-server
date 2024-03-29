use crate::actor::events::types::SimulatorParams;
use virus_simulator::Simulator;
use virus_simulator::State;

const POPULATION: f64 = 5000.0;
const TOTAL_DAYS: f64 = 700.0;

pub fn serialize_state(s: &[State], population: f64) -> String {
    // serilising the data
    let res = s
        .iter()
        .map(|state| {
            let values = state
                .iter()
                .enumerate()
                .map(|(ind, state)| {
                    if ind == 4 {
                        state.to_string()
                    } else {
                        (state * population).to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join(",");
            format!("[{}]", values)
        })
        .collect::<Vec<String>>()
        .join(",");
    format!("[{}]", res)
}

pub fn simulate(
    params: &SimulatorParams,
    changed_params: &[f64],
    cur_date: i32,
) -> (String, f64, f64, f64, f64) {
    let susceptible = params.susceptible / POPULATION;
    let exposed = params.exposed / POPULATION;
    let infectious = params.infectious / POPULATION;
    let removed = params.removed / POPULATION;

    let sim = Simulator::new(
        &susceptible,
        &exposed,
        &infectious,
        &removed,
        &params.current_reproduction_number,
        &changed_params[0],
        &changed_params[1],
        &changed_params[2],
        &changed_params[3],
    );

    let f = sim.simulate(0_f64, TOTAL_DAYS - cur_date as f64);
    (
        serialize_state(&f, POPULATION),
        susceptible,
        exposed,
        infectious,
        removed,
    )
}

#[macro_export]
macro_rules! zip {
    ($x: expr) => ($x);
    ($x: expr, $($y: expr), +) => (
        $x.iter().zip(
            zip!($($y), +))
    )
}

pub use zip;
