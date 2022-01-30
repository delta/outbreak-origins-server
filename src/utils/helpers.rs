use std::fmt::Write;
use virus_simulator::State;

pub fn writer(output: &mut String, s: &[State], population: f64) {
    // serilising the data
    output.write_str("{").unwrap();
    for (_j, state) in s.iter().enumerate() {
        let mut i = 1;
        output.write_str("[").unwrap();
        for val in state.iter() {
            if i % 5 == 0 {
                output.write_fmt(format_args!("{} , ", val)).unwrap();
            } else {
                output
                    .write_fmt(format_args!("{} , ", val * population))
                    .unwrap();
            }
            output.pop();
            output.pop();
            i += 1;
        }
        output.write_str("], ").unwrap();
        output.pop();
        output.pop();
        output.write_str(",").unwrap();
    }
    output.write_str("}").unwrap();
}
