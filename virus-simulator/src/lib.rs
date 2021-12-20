#![crate_name = "virus_simulator"]
use ode_solvers::dopri5::*;
use ode_solvers::*;

type State = Vector5<f64>;
type Time = f64;

/// Simulator based on SEIR Model
pub struct Simulator<'a> {
    susceptible: &'a f64,
    exposed: &'a f64,
    infectious: &'a f64,
    removed: &'a f64,
    current_reproduction_number: &'a f64,
    ideal_reproduction_number: &'a f64,
    compliance_factor: &'a f64,
    recovery_rate: &'a f64,
    infection_rate: &'a f64,
}

impl<'a> Simulator<'a> {
    pub fn new(
        susceptible: &'a f64,
        exposed: &'a f64,
        infectious: &'a f64,
        removed: &'a f64,
        current_reproduction_number: &'a f64,
        ideal_reproduction_number: &'a f64,
        compliance_factor: &'a f64,
        recovery_rate: &'a f64,
        infection_rate: &'a f64,
    ) -> Self {
        Self {
            susceptible,
            exposed,
            infectious,
            removed,
            current_reproduction_number,
            ideal_reproduction_number,
            compliance_factor,
            recovery_rate,
            infection_rate,
        }
    }
}

impl<'a> ode_solvers::System<State> for Simulator<'a> {
    fn system(&self, _t: Time, y: &State, dy: &mut State) {
        // y[0..4] represent S, E, I, R and current reproduction number respectively

        dy[0] = -self.recovery_rate * y[4] * y[0] * y[2];
        dy[1] = self.recovery_rate * y[4] * y[0] * y[2] - self.infection_rate * y[1];
        dy[2] = self.infection_rate * y[1] - self.recovery_rate * y[2];
        dy[3] = self.recovery_rate * y[2];
        dy[4] = self.compliance_factor * (self.ideal_reproduction_number - y[4]);
    }
}

// TODO: Write tests
