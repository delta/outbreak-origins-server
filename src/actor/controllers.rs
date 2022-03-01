use crate::actor::events::types::{
    ControlMeasure, ControlMeasureAction, ControlMeasureParams, Event, EventAction, EventParams,
    Save, Seed, SimulatorParams, SimulatorResponse, Start, StartParams, WSResponse,
};
use crate::db::models;
use diesel::prelude::*;
use diesel::PgConnection;

use crate::actor::utils::serialize_state;
use crate::auth::extractors;

use crate::db::types::DbError;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use virus_simulator::Simulator;

const POPULATION: f64 = 5000.0;
const TOTAL_DAYS: f64 = 700.0;
const EVENT_POSTPONE_PENALTY: i32 = 100;

impl Seed {
    pub fn handle(
        user: &extractors::Authenticated,
        conn: &PgConnection,
    ) -> Result<WSResponse, DbError> {
        use crate::db::schema::users::dsl::*;
        let user = user.0.as_ref().unwrap();
        let user = users
            .filter(email.eq(user.email.clone()))
            .first::<models::User>(conn)
            .optional()?;

        let user = match user {
            None => return Ok(WSResponse::Error("User not found".to_string())),
            Some(y) => y,
        };

        let file = format!("src/game/levels/{}/seed.json", user.curlevel);
        let path = Path::new(&file);
        let contents = fs::read_to_string(&path).expect("Something went wrong reading the file");
        Ok(WSResponse::Seed(contents))
    }
}

impl Start {
    pub fn handle(
        payload: String,
        user: &extractors::Authenticated,
        conn: &PgConnection,
    ) -> Result<WSResponse, DbError> {
        use crate::db::schema::status::dsl::*;
        use crate::db::schema::users;
        let user = user.0.as_ref().unwrap();
        let user = (users::table)
            .filter(users::email.eq(user.email.clone()))
            .first::<models::User>(conn)
            .optional()?;

        let user = match user {
            None => return Ok(WSResponse::Error("User not found".to_string())),
            Some(y) => y,
        };

        // Get the associated status entry for this user
        let user_status_id: i32 = match user.status {
            Some(s_id) => s_id,
            None => {
                let s_id = diesel::insert_into(status)
                    .default_values()
                    .get_result::<(i32, i32, i32, i32)>(conn)?
                    .0;

                diesel::update((users::table).filter(users::email.eq(user.email)))
                    .set(users::status.eq(s_id))
                    .execute(conn)?;
                s_id
            }
        };

        let region = serde_json::from_str::<Start>(&payload).unwrap().region;

        // Load the created regions for this user
        use crate::db::schema::regions;
        use crate::db::schema::regions_status;
        let region_ids = (regions_status::table)
            .filter(regions_status::status_id.eq(user_status_id))
            .select(regions_status::region_id)
            .load::<i32>(conn)?;
        let existing_regions = (regions::table)
            .filter(regions::id.eq_any(region_ids))
            .select((regions::id, regions::region_id))
            .load::<(i32, i32)>(conn)?;

        // Get the region id
        let mut first_time = false;
        let user_region_id: i32 = match existing_regions.into_iter().find(|&x| x.1 == region) {
            Some(r_s_tuple) => r_s_tuple.0,
            None => {
                // Initialise the region
                let new_region_id = diesel::insert_into(regions::table)
                    .values(regions::region_id.eq(&region))
                    .get_result::<(
                        i32,
                        i32,
                        SimulatorParams,
                        models::status::ActiveControlMeasures,
                    )>(conn)?
                    .0;

                // Create an entry in regions_status
                diesel::insert_into(regions_status::table)
                    .values((
                        regions_status::status_id.eq(user_status_id),
                        regions_status::region_id.eq(new_region_id),
                    ))
                    .execute(conn)?;

                first_time = true;
                new_region_id
            }
        };

        if first_time {
            let file = format!("src/game/levels/{}/start.json", user.curlevel);
            let path = Path::new(&file);
            let contents =
                fs::read_to_string(&path).expect("Something went wrong reading the file");
            let data = serde_json::from_str::<StartParams>(&contents).unwrap();

            match data.params.get(&region.to_string()) {
                Some(start_params) => {
                    // Update the status of this region
                    diesel::update(regions::table.filter(regions::id.eq(user_region_id)))
                        .set(regions::simulation_params.eq(start_params))
                        .execute(conn)?;

                    let sim = Simulator::new(
                        &start_params.susceptible,
                        &start_params.exposed,
                        &start_params.infectious,
                        &start_params.removed,
                        &start_params.current_reproduction_number,
                        &start_params.ideal_reproduction_number,
                        &start_params.compliance_factor,
                        &start_params.recovery_rate,
                        &start_params.infection_rate,
                    );
                    let f = sim.simulate(0_f64, TOTAL_DAYS);

                    let payload = serialize_state(&f, POPULATION);
                    Ok(WSResponse::Start(SimulatorResponse {
                        date: 0,
                        region,
                        payload,
                        ideal_reproduction_number: start_params.ideal_reproduction_number,
                        compliance_factor: start_params.compliance_factor,
                        recovery_rate: start_params.recovery_rate,
                        infection_rate: start_params.infection_rate,
                    }))
                }
                None => Ok(WSResponse::Error("Internal Server Error".to_string())),
            }
        } else {
            let sim_params = (regions::table)
                .filter(regions::id.eq(user_region_id))
                .select(regions::simulation_params)
                .first::<SimulatorParams>(conn)?;
            let sim = Simulator::new(
                &sim_params.susceptible,
                &sim_params.exposed,
                &sim_params.infectious,
                &sim_params.removed,
                &sim_params.current_reproduction_number,
                &sim_params.ideal_reproduction_number,
                &sim_params.compliance_factor,
                &sim_params.recovery_rate,
                &sim_params.infection_rate,
            );
            let f = sim.simulate(0_f64, TOTAL_DAYS);

            let payload = serialize_state(&f, POPULATION);

            let date = status
                .filter(id.eq(user_status_id))
                .select(cur_date)
                .first::<i32>(conn)?;

            Ok(WSResponse::Start(SimulatorResponse {
                date,
                region,
                payload,
                ideal_reproduction_number: sim_params.ideal_reproduction_number,
                compliance_factor: sim_params.compliance_factor,
                recovery_rate: sim_params.recovery_rate,
                infection_rate: sim_params.infection_rate,
            }))
        }
    }
}

impl ControlMeasure {
    pub fn handle(
        payload: String,
        user: &extractors::Authenticated,
        conn: &PgConnection,
    ) -> Result<WSResponse, DbError> {
        use crate::db::schema::{regions, regions_status, status, users};

        // User input wrapped in a Result
        let control_measure_request_result = serde_json::from_str::<ControlMeasure>(&payload);
        let user = user.0.as_ref().unwrap();
        let user = (users::table)
            .filter(users::email.eq(user.email.clone()))
            .first::<models::User>(conn)
            .optional()?;

        // Check if user present
        let user = match user {
            None => return Ok(WSResponse::Error("User not found".to_string())),
            Some(y) => y,
        };

        let res = match control_measure_request_result {
            Err(_) => Ok(WSResponse::Error("Couldn't parse request".to_string())),
            // If valid request
            Ok(control_measure_request) => {
                // Reads data from control measure file
                let file = format!("src/game/levels/{}/control.json", user.curlevel);
                let path = Path::new(&file);
                let contents = match fs::read_to_string(&path) {
                    Err(_) => return Ok(WSResponse::Error("Internal Server Error".to_string())),
                    Ok(val) => val,
                };

                // Set date in user status
                let status_id = match user.status {
                    Some(s_id) => s_id,
                    None => return Ok(WSResponse::Error("User status not found".to_string())),
                };

                let active_control_measure = (regions::table)
                    .inner_join(regions_status::table)
                    .filter(regions_status::status_id.eq(status_id))
                    .filter(regions::region_id.eq(control_measure_request.region as i32))
                    .select(regions::active_control_measures)
                    .load::<models::status::ActiveControlMeasures>(conn)?;

                if let Some(control) = active_control_measure[0]
                    .0
                    .get(&control_measure_request.name)
                {
                    if *control == control_measure_request.level {
                        return Ok(WSResponse::Error(
                            "Control measure with same level already active".to_string(),
                        ));
                    }
                }

                diesel::update(status::table)
                    .filter(status::id.eq(status_id))
                    .set(status::cur_date.eq(control_measure_request.cur_date))
                    .execute(conn)?;

                let control_measure_data =
                    serde_json::from_str::<HashMap<String, ControlMeasureParams>>(&contents)
                        .unwrap();

                let zero_delta: &Vec<f64> = &vec![0_f64; 4];

                let (mut active_control_measures, existing_delta) = if active_control_measure
                    .is_empty()
                {
                    let regions_row = diesel::insert_into(regions::table)
                        .values(regions::region_id.eq(control_measure_request.region as i32))
                        .get_result::<(
                            i32,
                            i32,
                            SimulatorParams,
                            models::status::ActiveControlMeasures,
                        )>(conn)?;

                    diesel::insert_into(regions_status::table)
                        .values((
                            regions_status::status_id.eq(status_id),
                            regions_status::region_id.eq(regions_row.0),
                        ))
                        .execute(conn)?;
                    (regions_row.3 .0, zero_delta)
                } else {
                    match active_control_measure[0]
                        .0
                        .get(&control_measure_request.name)
                    {
                        Some(val) => {
                            match control_measure_data
                                .get(&control_measure_request.name)
                                .unwrap()
                                .levels
                                .get(val)
                            {
                                Some(y) => (active_control_measure[0].0.clone(), &y.params_delta),
                                // Should never happen since the user can only have
                                // a control measure if present in our file
                                None => {
                                    return Ok(WSResponse::Error(
                                        "Internal Server Error".to_string(),
                                    ))
                                }
                            }
                        }
                        None => (active_control_measure[0].0.clone(), zero_delta),
                    }
                };

                match control_measure_data.get(&control_measure_request.name) {
                    Some(control_measure_params) => match control_measure_params
                        .levels
                        .get(&control_measure_request.level)
                    {
                        Some(control_measure_level_info) => {
                            if control_measure_request.action == ControlMeasureAction::Apply
                                && control_measure_level_info.cost > user.money as u32
                            {
                                return Ok(WSResponse::Error("Not enough money".to_string()));
                            }
                            if control_measure_request.action == ControlMeasureAction::Remove
                                && existing_delta == zero_delta
                            {
                                return Ok(WSResponse::Error(
                                    "Control measure was not applied".to_string(),
                                ));
                            }

                            let target_delta: &Vec<f64> = match control_measure_request.action {
                                ControlMeasureAction::Apply => {
                                    if let Some(x) = active_control_measures
                                        .get_mut(&control_measure_request.name)
                                    {
                                        *x = control_measure_request.level;
                                    } else {
                                        active_control_measures.insert(
                                            control_measure_request.name,
                                            control_measure_request.level,
                                        );
                                    }
                                    &control_measure_level_info.params_delta
                                }
                                ControlMeasureAction::Remove => {
                                    active_control_measures.remove(&control_measure_request.name);
                                    zero_delta
                                }
                            };

                            let net_delta: Vec<f64> = existing_delta
                                .iter()
                                .zip(target_delta.iter())
                                .map(|(a, b)| b - a)
                                .collect();

                            let recvd_params = [
                                control_measure_request.params.ideal_reproduction_number,
                                control_measure_request.params.compliance_factor,
                                control_measure_request.params.recovery_rate,
                                control_measure_request.params.infection_rate,
                            ];

                            let changed_params: Vec<f64> = net_delta
                                .iter()
                                .zip(recvd_params.iter())
                                .map(|(&a, &b)| a + b)
                                .collect();

                            let susceptible =
                                control_measure_request.params.susceptible / POPULATION;
                            let exposed = control_measure_request.params.exposed / POPULATION;
                            let infectious = control_measure_request.params.infectious / POPULATION;
                            let removed = control_measure_request.params.removed / POPULATION;
                            let sim = Simulator::new(
                                &susceptible,
                                &exposed,
                                &infectious,
                                &removed,
                                &control_measure_request.params.current_reproduction_number,
                                &changed_params[0],
                                &changed_params[1],
                                &changed_params[2],
                                &changed_params[3],
                            );

                            let f = sim.simulate(
                                0_f64,
                                TOTAL_DAYS - control_measure_request.cur_date as f64,
                            );
                            let payload = serialize_state(&f, POPULATION);

                            conn.transaction::<_, diesel::result::Error, _>(|| {
                                diesel::update(regions::table)
                                    .filter(
                                        regions::id.eq_any(
                                            regions_status::table
                                                .filter(regions_status::status_id.eq(status_id))
                                                .select(regions_status::region_id)
                                                .load::<i32>(conn)?,
                                        ),
                                    )
                                    .filter(
                                        regions::region_id
                                            .eq(control_measure_request.region as i32),
                                    )
                                    .set((
                                        regions::active_control_measures.eq(
                                            models::status::ActiveControlMeasures(
                                                active_control_measures,
                                            ),
                                        ),
                                        regions::simulation_params.eq(SimulatorParams {
                                            susceptible,
                                            exposed,
                                            infectious,
                                            removed,
                                            current_reproduction_number: control_measure_request
                                                .params
                                                .current_reproduction_number,
                                            ideal_reproduction_number: changed_params[0],
                                            compliance_factor: changed_params[1],
                                            recovery_rate: changed_params[2],
                                            infection_rate: changed_params[3],
                                        }),
                                    ))
                                    .execute(conn)?;
                                diesel::update(users::table)
                                        .filter(users::email.eq(user.email))
                                        .set(
                                            users::money
                                                .eq(user.money
                                                    - control_measure_level_info.cost as i32),
                                        )
                                        .execute(conn)?;
                                Ok(())
                            })?;
                            Ok(WSResponse::Control(SimulatorResponse {
                                date: control_measure_request.cur_date,
                                region: control_measure_request.region as i32,
                                payload,
                                ideal_reproduction_number: changed_params[0],
                                compliance_factor: changed_params[1],
                                recovery_rate: changed_params[2],
                                infection_rate: changed_params[3],
                            }))
                        }
                        None => Ok(WSResponse::Error("Level not found".to_string())),
                    },
                    None => Ok(WSResponse::Error("No control measure found".to_string())),
                }
            }
        };
        res
    }
}

impl Event {
    pub fn handle(
        payload: String,
        user: &extractors::Authenticated,
        conn: &PgConnection,
    ) -> Result<WSResponse, DbError> {
        use crate::db::schema::users::dsl::*;
        let user = user.0.as_ref().unwrap();
        let user = users
            .filter(email.eq(user.email.clone()))
            .first::<models::User>(conn)
            .optional()?;
        let user = match user {
            None => return Ok(WSResponse::Error("User not found".to_string())),
            Some(y) => y,
        };

        let user_status_id = match user.status {
            Some(x) => x,
            None => return Ok(WSResponse::Error("Internal Server Error".to_string())),
        };

        let event = serde_json::from_str::<Event>(&payload);

        match event {
            Err(_) => Ok(WSResponse::Error("Couldn't parse request".to_string())),

            Ok(event) => {
                let file = format!("src/game/levels/{}/event.json", user.curlevel);
                let path = Path::new(&file);
                let contents =
                    fs::read_to_string(&path).expect("Something  went wrong reading the file");
                let event_data =
                    serde_json::from_str::<HashMap<String, EventParams>>(&contents).unwrap();

                use crate::db::schema::status::dsl::*;

                diesel::update(status)
                    .filter(id.eq(user_status_id))
                    .set(cur_date.eq(event.cur_date))
                    .execute(conn)?;

                match event.action {
                    EventAction::Request => match status
                        .filter(id.eq(user_status_id))
                        .select(current_event)
                        .first::<i32>(conn)
                    {
                        Ok(event_id) => {
                            if event_id == 0 {
                                diesel::update(status)
                                    .filter(id.eq(user_status_id))
                                    .set((current_event.eq(1), postponed.eq(0)))
                                    .execute(conn)?;
                                match event_data.get("1") {
                                    Some(data) => Ok(WSResponse::EventParams(EventParams {
                                        id: event_id,
                                        name: data.name.clone(),
                                        description: data.description.clone(),
                                        params_delta: data.params_delta.clone(),
                                        region: data.region,
                                        reward: data.reward,
                                    })),
                                    None => {
                                        Ok(WSResponse::Error("Couldn't read the file".to_string()))
                                    }
                                }
                            } else {
                                match event_data.get(&event_id.to_string()) {
                                    Some(data) => Ok(WSResponse::EventParams(EventParams {
                                        id: event_id,
                                        name: data.name.clone(),
                                        description: data.description.clone(),
                                        params_delta: data.params_delta.clone(),
                                        region: data.region,
                                        reward: data.reward,
                                    })),
                                    None => {
                                        Ok(WSResponse::Error("Couldn't read the file".to_string()))
                                    }
                                }
                            }
                        }
                        Err(_) => Ok(WSResponse::Error("Couldn't find user".to_string())),
                    },
                    EventAction::Accept => match event_data.get(&event.id.to_string()) {
                        Some(data) => {
                            let user_status =
                                status
                                    .filter(id.eq(user_status_id))
                                    .first::<(i32, i32, i32, i32)>(conn)?;
                            let reward = if user_status.1 != event.id {
                                return Ok(WSResponse::Error(
                                    "Cannot Accept event which wasn't requested".to_string(),
                                ));
                            } else {
                                data.reward - user_status.2 * EVENT_POSTPONE_PENALTY
                            };

                            diesel::update(users.filter(email.eq(user.email)))
                                .set(money.eq(money + reward))
                                .execute(conn)?;

                            diesel::update(status)
                                .filter(id.eq(user_status_id))
                                .set((current_event.eq(current_event + 1), postponed.eq(0)))
                                .execute(conn)?;

                            let recvd_params = [
                                event.params.ideal_reproduction_number,
                                event.params.compliance_factor,
                                event.params.recovery_rate,
                                event.params.infection_rate,
                            ];

                            let changed_params: Vec<f64> = data
                                .params_delta
                                .iter()
                                .zip(recvd_params.iter())
                                .map(|(&a, &b)| a + b)
                                .collect();

                            let susceptible = event.params.susceptible / POPULATION;
                            let exposed = event.params.exposed / POPULATION;
                            let infectious = event.params.infectious / POPULATION;
                            let removed = event.params.removed / POPULATION;
                            let sim = Simulator::new(
                                &susceptible,
                                &exposed,
                                &infectious,
                                &removed,
                                &event.params.current_reproduction_number,
                                &changed_params[0],
                                &changed_params[1],
                                &changed_params[2],
                                &changed_params[3],
                            );

                            let f = sim.simulate(0_f64, TOTAL_DAYS - event.cur_date as f64);
                            let payload = serialize_state(&f, POPULATION);

                            Ok(WSResponse::Event(SimulatorResponse {
                                date: event.cur_date,
                                region: data.region,
                                payload,
                                ideal_reproduction_number: changed_params[0],
                                compliance_factor: changed_params[1],
                                recovery_rate: changed_params[2],
                                infection_rate: changed_params[3],
                            }))
                        }
                        None => Ok(WSResponse::Error("Invalid request sent".to_string())),
                    },
                    EventAction::Decline => {
                        diesel::update(status)
                            .filter(id.eq(user_status_id))
                            .set((current_event.eq(current_event + 1), postponed.eq(0)))
                            .execute(conn)?;
                        Ok(WSResponse::Ok("Declined".to_string()))
                    }
                    EventAction::Postpone => {
                        diesel::update(status)
                            .filter(id.eq(user_status_id))
                            .set(postponed.eq(postponed + 1))
                            .execute(conn)?;
                        Ok(WSResponse::Ok("Postponed".to_string()))
                    }
                }
            }
        }
    }
}

impl Save {
    pub fn handle(
        payload: String,
        user: &extractors::Authenticated,
        conn: &PgConnection,
    ) -> Result<WSResponse, DbError> {
        use crate::db::schema::{regions, regions_status, users};
        let save_request_result = serde_json::from_str::<Save>(&payload);
        let user = user.0.as_ref().unwrap();
        let user = (users::table)
            .filter(users::email.eq(user.email.clone()))
            .first::<models::User>(conn)
            .optional()?;
        let user = match user {
            None => return Ok(WSResponse::Error("User not found".to_string())),
            Some(y) => y,
        };

        match save_request_result {
            Err(_) => Ok(WSResponse::Error("Couldn't parse request".to_string())),
            Ok(save_request) => {
                let status_id = match user.status {
                    Some(status_id) => status_id,
                    None => return Ok(WSResponse::Error("Internal Server Error".to_string())),
                };

                diesel::update(regions::table)
                    .set(regions::simulation_params.eq(save_request.params))
                    .filter(regions::region_id.eq(save_request.region as i32))
                    .filter(
                        regions::id.eq_any(
                            regions_status::table
                                .filter(regions_status::status_id.eq(status_id))
                                .select(regions_status::region_id),
                        ),
                    )
                    .execute(conn)?;
                Ok(WSResponse::Ok("Saved".to_string()))
            }
        }
    }
}
