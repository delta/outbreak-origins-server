use crate::actor::events::utils::enum_str;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

// Event types
#[derive(Serialize)]
pub struct NewsEvent {
    pub img: String,
    pub heading: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct StartEvent {
    pub payload: String,
}

#[derive(Serialize, Deserialize)]
pub struct SentEvent {
    event_type: String,
    payload: String,
}

enum_str!(
    enum EventType {
        // News(NewsEvent),
        Start(StartEvent),
    }
);

#[derive(Display)]
pub enum ReceivedEvent {
    // News,
    Start,
}
