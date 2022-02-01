<<<<<<< HEAD
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct StartLevelRequest {
    pub level: i32,
}

#[derive(Serialize)]
pub struct StartLevelError {
    pub message: String,
}
=======

>>>>>>> a13592d (feat: Adds start-level route)
