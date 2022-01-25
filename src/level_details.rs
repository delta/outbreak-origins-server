pub struct LevelDetails {
    pub level: i32,
    pub initial_susceptible: f64,
    pub initial_exposed: f64,
    pub initial_infected: f64,
    pub initial_removed: f64,
    pub initial_reproduction_number: f64,
    pub initial_ideal_reproduction_number: f64,
    pub initial_infection_rate: f64,
    pub initial_recovery_rate: f64,
    pub initial_social_parameter: f64,
}

#[allow(clippy::too_many_arguments)]
impl LevelDetails {
    const fn new(
        level: i32,
        initial_susceptible: f64,
        initial_exposed: f64,
        initial_infected: f64,
        initial_removed: f64,
        initial_reproduction_number: f64,
        initial_ideal_reproduction_number: f64,
        initial_infection_rate: f64,
        initial_recovery_rate: f64,
        initial_social_parameter: f64,
    ) -> Self {
        Self {
            level,
            initial_susceptible,
            initial_exposed,
            initial_infected,
            initial_removed,
            initial_reproduction_number,
            initial_ideal_reproduction_number,
            initial_infection_rate,
            initial_recovery_rate,
            initial_social_parameter,
        }
    }
}

pub static LEVEL1: LevelDetails = LevelDetails::new(
    1,
    0.9999995,
    0.0000004,
    0.0000001,
    0.0,
    1.6,
    2.0,
    0.0555,
    0.1923076923,
    0.5,
);

pub static LEVEL2: LevelDetails = LevelDetails::new(
    2,
    0.9999994,
    0.0000005,
    0.0000001,
    0.0,
    1.6,
    2.0,
    0.0555,
    0.1923076923,
    0.5,
);

pub static LEVEL3: LevelDetails = LevelDetails::new(
    3,
    0.9999993,
    0.0000006,
    0.0000001,
    0.0,
    1.6,
    2.0,
    0.0555,
    0.1923076923,
    0.5,
);

pub static LEVEL4: LevelDetails = LevelDetails::new(
    4,
    0.9999992,
    0.0000007,
    0.0000001,
    0.0,
    1.6,
    2.0,
    0.0555,
    0.1923076923,
    0.5,
);
