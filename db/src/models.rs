#[derive(Queryable)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub reward: i32,
    pub current_state: i32,
    pub compliance_reward: f64,
    pub infection_rate: f64,
    pub next_event_time: i32,
}
