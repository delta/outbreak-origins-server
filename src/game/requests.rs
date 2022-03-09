use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChangeLevelRequest {
    pub is_randomized: bool,
}

#[derive(Deserialize)]
pub struct LevelRequest {
    pub level: i32,
}

#[derive(Deserialize, Debug)]
pub struct StartLevelRequest {
    pub level: i32,
}

#[derive(Deserialize, Debug)]
pub struct EndLevelRequest {
    pub infected: f64,
    pub removed: f64,
    pub money_left: f64,
}
