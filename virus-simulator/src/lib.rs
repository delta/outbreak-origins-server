use ode_solvers::dopri5::*;
use ode_solvers::*;

type State<'a> = Vector5<&'a f64>;
type Time = f64;

pub struct Simulator<'a> {
    susceptible: &'a f64,
    exposed: &'a f64,
    infectious: &'a f64,
    removed: &'a f64,
    ideal_reproduction_number: &'a f64,
    current_reproduction_number: &'a f64,
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
        ideal_reproduction_number: &'a f64,
        current_reproduction_number: &'a f64,
        compliance_factor: &'a f64,
        recovery_rate: &'a f64,
        infection_rate: &'a f64,
    ) -> Self {
        Self {
            susceptible,
            exposed,
            infectious,
            removed,
            ideal_reproduction_number,
            current_reproduction_number,
            compliance_factor,
            recovery_rate,
            infection_rate,
        }
    }

}

impl<'a> ode_solvers::System<State<'a>> for Simulator<'a> {
    fn system(&self, _t: Time, y: &State, dy: &mut State) {
        let self.susceptible = y[0];
        let self.exposed = y[1];
        let self.infectious = y[2];
        let self.removed = y[3];
        let self.current_reproduction_number = y[4];

        dy[0] = -self.recovery_rate * current_reproduction_number * susceptible * infectious;
        dy[1] = self.recovery_rate * current_reproduction_number * susceptible * infectious
            - self.infection_rate * exposed;
        dy[2] = self.infection_rate * self.exposed - self.recovery_rate * infectious;
        dy[3] = self.recovery_rate * self.infectious;
        dy[4] = self.social_parameter * (self.ideal_reproduction_number - self.reproduction_number);
    }
}

// TODO: Write tests