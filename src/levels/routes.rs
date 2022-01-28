use crate::levels::response;
use actix_web::{post, web, Error, HttpResponse};
use std::fs::File;

#[post("/dashboard")]
async fn level_details(level: web::Json<response::LevelInfo>) -> Result<HttpResponse, Error> {
    let mut file = File::open("src/game/levelDetails.json").unwrap();
    let json: response::Levels = serde_json::from_reader(&mut file).unwrap();
    match level.level {
        i @ (1..=4) => Ok(HttpResponse::Ok().json(response::LevelResult {
            initial_susceptible: json.levels[i - 1].level.initial_susceptible,
            initial_exposed: json.levels[i - 1].level.initial_exposed,
            initial_infected: json.levels[i - 1].level.initial_infected,
            initial_removed: json.levels[i - 1].level.initial_removed,
            initial_reproduction_number: json.levels[i - 1].level.initial_reproduction_number,
            initial_ideal_reproduction_number: json.levels[i - 1]
                .level
                .initial_ideal_reproduction_number,
            initial_infection_rate: json.levels[i - 1].level.initial_infection_rate,
            initial_recovery_rate: json.levels[i - 1].level.initial_recovery_rate,
            initial_social_parameter: json.levels[i - 1].level.initial_social_parameter,
        })),
        _ => Ok(HttpResponse::InternalServerError().finish()),
    }
}

pub fn game_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/user/api").service(level_details));
}
