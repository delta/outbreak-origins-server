use virus_simulator::State;

pub fn serialize_state(s: &[State], population: f64) -> String {
    // serilising the data
    let res = s
        .iter()
        .map(|state| {
            let values = state
                .iter()
                .enumerate()
                .map(|(ind, state)| {
                    if ind == 5 {
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
