use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChangeLevelRequest {
    pub is_randomized: bool,
}
