use crate::db::models::status::ActiveControlMeasures;
use std::collections::HashMap;

pub fn get_acm_map_from_db_res(
    acm_tuples: &[(i32, ActiveControlMeasures)],
) -> HashMap<String, Vec<ActiveControlMeasures>> {
    let mut acm_map = HashMap::<String, Vec<ActiveControlMeasures>>::new();
    for (k, v) in acm_tuples {
        acm_map
            .entry(k.to_string())
            .or_insert_with(Vec::new)
            .push(v.clone())
    }
    acm_map
}
