use crate::actor::events::utils::enum_str;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

// Event types
#[derive(Serialize)]
pub struct NewsResponse {
    pub img: String,
    pub heading: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct StartResponse {
    pub payload: String,
}

#[derive(Serialize, Deserialize)]
pub struct WSPayload {
    event_type: String,
    payload: String,
}

enum_str!(
    enum WSResponse {
        // News(NewsEvent),
        Start(StartResponse),
    }
);

#[derive(Display)]
pub enum WSRequest {
    // News,
    Start,
}
