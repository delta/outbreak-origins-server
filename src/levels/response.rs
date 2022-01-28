use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LevelRequest {
    pub level: usize,
}

#[derive(Deserialize)]
pub struct LevelDetails {
    pub level: LevelResponse,
}

#[derive(Deserialize)]
pub struct Levels {
    pub levels: Vec<LevelDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelResponse {
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

#[derive(Debug, Clone, Serialize)]
pub struct LevelError {
    message: String,
}
