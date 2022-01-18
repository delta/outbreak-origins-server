#[derive(Queryable)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub description: String,

    pub compliance_factor: f64,
    pub infection_rate: f64,
    pub ideal_reproduction_number: f64,
}
