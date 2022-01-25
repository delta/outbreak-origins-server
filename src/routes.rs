use crate::level_details::{LEVEL1, LEVEL2, LEVEL3, LEVEL4};
use actix_web::{post, web, Error, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct LevelInfo {
    level: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LevelResult {
    initial_susceptible: f64,
    initial_exposed: f64,
    initial_infected: f64,
    initial_removed: f64,
    initial_reproduction_number: f64,
    initial_ideal_reproduction_number: f64,
    initial_infection_rate: f64,
    initial_recovery_rate: f64,
    initial_social_parameter: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LevelError {
    message: String,
}

#[post("/dashboard")]
async fn level_details(level: web::Json<LevelInfo>) -> Result<HttpResponse, Error> {
    match level.level {
        1 => Ok(HttpResponse::Ok().json(LevelResult {
            initial_susceptible: LEVEL1.initial_susceptible,
            initial_exposed: LEVEL1.initial_exposed,
            initial_infected: LEVEL1.initial_infected,
            initial_removed: LEVEL1.initial_removed,
            initial_reproduction_number: LEVEL1.initial_reproduction_number,
            initial_ideal_reproduction_number: LEVEL1.initial_ideal_reproduction_number,
            initial_infection_rate: LEVEL1.initial_infection_rate,
            initial_recovery_rate: LEVEL1.initial_recovery_rate,
            initial_social_parameter: LEVEL1.initial_social_parameter,
        })),
        2 => Ok(HttpResponse::Ok().json(LevelResult {
            initial_susceptible: LEVEL2.initial_susceptible,
            initial_exposed: LEVEL2.initial_exposed,
            initial_infected: LEVEL2.initial_infected,
            initial_removed: LEVEL2.initial_removed,
            initial_reproduction_number: LEVEL2.initial_reproduction_number,
            initial_ideal_reproduction_number: LEVEL2.initial_ideal_reproduction_number,
            initial_infection_rate: LEVEL2.initial_infection_rate,
            initial_recovery_rate: LEVEL2.initial_recovery_rate,
            initial_social_parameter: LEVEL2.initial_social_parameter,
        })),
        3 => Ok(HttpResponse::Ok().json(LevelResult {
            initial_susceptible: LEVEL3.initial_susceptible,
            initial_exposed: LEVEL3.initial_exposed,
            initial_infected: LEVEL3.initial_infected,
            initial_removed: LEVEL3.initial_removed,
            initial_reproduction_number: LEVEL3.initial_reproduction_number,
            initial_ideal_reproduction_number: LEVEL3.initial_ideal_reproduction_number,
            initial_infection_rate: LEVEL3.initial_infection_rate,
            initial_recovery_rate: LEVEL3.initial_recovery_rate,
            initial_social_parameter: LEVEL3.initial_social_parameter,
        })),
        4 => Ok(HttpResponse::Ok().json(LevelResult {
            initial_susceptible: LEVEL4.initial_susceptible,
            initial_exposed: LEVEL4.initial_exposed,
            initial_infected: LEVEL4.initial_infected,
            initial_removed: LEVEL4.initial_removed,
            initial_reproduction_number: LEVEL4.initial_reproduction_number,
            initial_ideal_reproduction_number: LEVEL4.initial_ideal_reproduction_number,
            initial_infection_rate: LEVEL4.initial_infection_rate,
            initial_recovery_rate: LEVEL4.initial_recovery_rate,
            initial_social_parameter: LEVEL4.initial_social_parameter,
        })),
        _ => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub fn game_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/user/api").service(level_details));
}
