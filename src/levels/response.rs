use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LevelRequest {
<<<<<<< HEAD
    pub level: i32,
=======
    pub level: usize,
>>>>>>> d620c4c (refactor: better names for Levels Struct)
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
>>>>>>> dc66ab7 (refactor: refactors levels backend)
}

#[derive(Debug, Clone, Serialize)]
pub struct LevelError {
    message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LevelResponse {
    pub cur_level: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DbResponse {
    pub message: String,
}
