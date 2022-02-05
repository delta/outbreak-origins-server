use serde::{Deserialize, Serialize};

// Simulation Types
#[derive(Serialize, Deserialize)]
pub struct InitParams {
    pub num_sections: i32,
    pub section_data: Vec<SectionData>,
}

#[derive(Serialize, Deserialize)]
pub struct SectionData {
    pub population: f64,
    pub init_params: InitSimulationParams,
}

#[derive(Serialize, Deserialize)]
pub struct InitSimulationParams {
    pub susceptible: f64,
    pub exposed: f64,
    pub infectious: f64,
    pub removed: f64,
    pub current_reproduction_number: f64,
    pub ideal_reproduction_number: f64,
    pub compliance_factor: f64,
    pub recovery_rate: f64,
    pub infection_rate: f64,
}

#[derive(Deserialize)]
pub struct ParamsDelta {
    pub name: String,
    pub value: f64,
}

#[derive(Deserialize)]
pub struct ControlMeasureParams {
    pub description: String,
    pub params_delta: Vec<ParamsDelta>,
}

#[derive(Deserialize)]
pub struct EventParam {
    pub description: String,
    pub params: Vec<ParamsDelta>,
    pub reward: i32,
}
