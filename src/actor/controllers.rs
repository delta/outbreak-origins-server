use crate::actor::events::types::{
    ActionResponse, ControlMeasure, ControlMeasureAction, ControlMeasureParams, Event, EventAction,
    EventParams, Read, Save, Seed, SimulatorParams, SimulatorResponse, Start, StartParams,
    WSResponse,
};
use crate::db::models;
use diesel::prelude::*;
use diesel::PgConnection;

use crate::actor::utils::{serialize_state, simulate, zip};
use crate::auth::extractors;

use crate::db::types::DbError;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use virus_simulator::Simulator;

use tracing::{error, info, instrument};

const POPULATION: f64 = 5000.0;
const TOTAL_DAYS: f64 = 700.0;
const EVENT_POSTPONE_PENALTY: i32 = 100;

const PARAM_LIMITS: &'static [(f64, f64)] = &[(1.2, 3.0), (0.0, 0.8), (0.05, 0.1), (0.05, 0.30)];

pub fn get_description(key: String, level: i32) -> Read {
    let file = format!("src/game/levels/{}/description.json", level);
    let path = Path::new(&file);
    let contents = fs::read_to_string(&path).expect("Something went wrong reading the file");
    let obj = serde_json::from_str::<HashMap<String, Read>>(&contents).unwrap();
    obj.get(&key).expect("Invalid key").clone()
}

impl Seed {
    #[instrument(skip(conn))]
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
        let contents = fs::read_to_string(&path).unwrap_or_else(|e| {
            error!("Couldn't read seed file: {}", e);
            String::default()
        });
        if contents == String::default() {
            return Ok(WSResponse::Error("Internal Server Error".to_string()));
        }
        Ok(WSResponse::Seed(contents))
    }
}

impl Start {
    #[instrument(skip(conn))]
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

                info!("Creating a status entry with id: {}", s_id);

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

                info!("Creating new region entry with id: {}", new_region_id);

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

                    // simulate(&start_params)
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

                    info!("Simulating Start with params: {:?}", start_params);
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

            info!("Simulating Start with params: {:?}", sim_params);
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
    #[instrument(skip(conn))]
    pub fn handle(
        payload: String,
        user: &extractors::Authenticated,
        conn: &PgConnection,
    ) -> Result<WSResponse, DbError> {
        use crate::db::schema::{regions, regions_status, status, users};
        use rand::{thread_rng, Rng};

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
                let control_measure_name =
                    &get_description(control_measure_request.name.clone(), user.curlevel);
                let mut control_measure_message = match control_measure_name {
                    Read::ControlNews(x) => x.apply.to_string(),
                    _ => "Invalid control measure".to_string(),
                };
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

                let control_measure_data =
                    serde_json::from_str::<HashMap<String, ControlMeasureParams>>(&contents)
                        .unwrap();

                let control_measure_failed = if user.is_randomized {
                    let mut rng = thread_rng();
                    let n: f32 = rng.gen_range(0.0..=1.0);
                    if let Some(val) = control_measure_data.get(&control_measure_request.name) {
                        n <= val.mess_up_chance
                    } else {
                        return Ok(WSResponse::Error("Invalid event".to_string()));
                    }
                } else {
                    false
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
                        info!("Control measure already applied at this level");
                        return Ok(WSResponse::Error(
                            "Control measure with same level already active".to_string(),
                        ));
                    }
                }

                diesel::update(status::table)
                    .filter(status::id.eq(status_id))
                    .set(status::cur_date.eq(control_measure_request.cur_date))
                    .execute(conn)?;

                let zero_delta: &Vec<f64> = &vec![0_f64; 4];

                let (mut active_control_measures, existing_delta) = if active_control_measure
                    .is_empty()
                {
                    let active_control_measures = if !control_measure_failed {
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
                        regions_row.3 .0
                    } else {
                        HashMap::<String, i32>::new()
                    };
                    (active_control_measures, zero_delta)
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

                let (target_delta, cost) = match control_measure_request.action {
                    ControlMeasureAction::Apply => {
                        match control_measure_data.get(&control_measure_request.name) {
                            Some(control_measure_params) => match control_measure_params
                                .levels
                                .get(&control_measure_request.level)
                            {
                                Some(control_measure_level_info) => {
                                    if control_measure_level_info.cost > user.money as u32 {
                                        info!("Not enough money");
                                        return Ok(WSResponse::Error(
                                            "Not enough money".to_string(),
                                        ));
                                    }
                                    let target = if !control_measure_failed {
                                        if let Some(x) = active_control_measures
                                            .get_mut(&control_measure_request.name)
                                        {
                                            *x = control_measure_request.level;
                                        } else {
                                            active_control_measures.insert(
                                                control_measure_request.name.clone(),
                                                control_measure_request.level,
                                            );
                                        }
                                        control_measure_level_info.params_delta.clone()
                                    } else {
                                        let target = control_measure_level_info
                                            .params_delta
                                            .iter()
                                            .enumerate()
                                            .map(|(ind, x)| match ind {
                                                0 => -x,
                                                1 => -x.abs(),
                                                2 => -x.abs(),
                                                3 => x.abs(),
                                                _ => unreachable!(),
                                            })
                                            .collect::<Vec<f64>>();
                                        target
                                    };
                                    (target, control_measure_level_info.cost)
                                }
                                None => {
                                    return Ok(WSResponse::Error("Level not found".to_string()));
                                }
                            },
                            None => {
                                return Ok(WSResponse::Error(
                                    "Control Measure not found".to_string(),
                                ));
                            }
                        }
                    }
                    ControlMeasureAction::Remove => {
                        if existing_delta == zero_delta {
                            return Ok(WSResponse::Error(
                                "Control Measure was not applied".to_string(),
                            ));
                        }

                        control_measure_message = match control_measure_name {
                            Read::ControlNews(x) => x.remove.to_string(),
                            _ => "Invalid control measure".to_string(),
                        };
                        active_control_measures.remove(&control_measure_request.name);
                        (zero_delta.to_vec(), 0)
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

                let zipped = zip!(net_delta, recvd_params, PARAM_LIMITS);
                let changed_params: Vec<f64> = zipped
                    .map(|(&a, (&b, &c))| {
                        if a + b < c.0 {
                            c.0
                        } else if a + b > c.1 {
                            c.1
                        } else {
                            a + b
                        }
                    })
                    .collect();

                info!(
                    "Simulating Control Measure with params: {:?}\n{:?}",
                    &control_measure_request.params, &changed_params
                );
                let (payload, susceptible, exposed, infectious, removed) = simulate(
                    &control_measure_request.params,
                    &changed_params,
                    control_measure_request.cur_date,
                );

                conn.transaction::<_, diesel::result::Error, _>(|| {
                    if !control_measure_failed {
                        diesel::update(regions::table)
                            .filter(
                                regions::id.eq_any(
                                    regions_status::table
                                        .filter(regions_status::status_id.eq(status_id))
                                        .select(regions_status::region_id)
                                        .load::<i32>(conn)?,
                                ),
                            )
                            .filter(regions::region_id.eq(control_measure_request.region as i32))
                            .set((
                                regions::active_control_measures.eq(
                                    models::status::ActiveControlMeasures(active_control_measures),
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
                    }
                    diesel::update(users::table)
                        .filter(users::email.eq(user.email))
                        .set(users::money.eq(user.money - cost as i32))
                        .execute(conn)?;
                    Ok(())
                })?;
                Ok(WSResponse::Control(ActionResponse {
                    simulation_data: SimulatorResponse {
                        date: control_measure_request.cur_date,
                        region: control_measure_request.region as i32,
                        payload,
                        ideal_reproduction_number: changed_params[0],
                        compliance_factor: changed_params[1],
                        recovery_rate: changed_params[2],
                        infection_rate: changed_params[3],
                    },
                    description: control_measure_message,
                    is_success: !control_measure_failed,
                }))
            }
        };
        res
    }
}

impl Event {
    #[instrument(skip(conn))]
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
                            info!("Requested Event");
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
                    EventAction::Accept => {
                        info!("Accepting Event: {}", &event.id);

                        let event_accept_message =
                            &get_description(event.id.to_string(), user.curlevel);
                        let event_accept_message = match event_accept_message {
                            Read::EventNews(x) => x.accept.to_string(),
                            _ => "Invalid Event".to_string(),
                        };
                        match event_data.get(&event.id.to_string()) {
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

                                let recvd_params = [
                                    event.params.ideal_reproduction_number,
                                    event.params.compliance_factor,
                                    event.params.recovery_rate,
                                    event.params.infection_rate,
                                ];

                                let zipped = zip!(data.params_delta, recvd_params, PARAM_LIMITS);
                                let changed_params: Vec<f64> = zipped
                                    .map(|(&a, (&b, &c))| {
                                        if a + b < c.0 {
                                            c.0
                                        } else if a + b > c.1 {
                                            c.1
                                        } else {
                                            a + b
                                        }
                                    })
                                    .collect();

                                info!(
                                    "Simulating Event with params: {:?}\n{:?}",
                                    &event.params, &changed_params
                                );
                                let (payload, susceptible, exposed, infectious, removed) =
                                    simulate(&event.params, &changed_params, event.cur_date);

                                conn.transaction::<_, diesel::result::Error, _>(|| {
                                    use crate::db::schema::{
                                        regions, regions_status, status, users,
                                    };
                                    diesel::update(regions::table)
                                        .filter(
                                            regions::id.eq_any(
                                                regions_status::table
                                                    .filter(
                                                        regions_status::status_id
                                                            .eq(user_status_id),
                                                    )
                                                    .select(regions_status::region_id)
                                                    .load::<i32>(conn)?,
                                            ),
                                        )
                                        .filter(regions::region_id.eq(data.region))
                                        .set(regions::simulation_params.eq(SimulatorParams {
                                            susceptible,
                                            exposed,
                                            infectious,
                                            removed,
                                            current_reproduction_number:
                                                event.params.current_reproduction_number,
                                            ideal_reproduction_number: changed_params[0],
                                            compliance_factor: changed_params[1],
                                            recovery_rate: changed_params[2],
                                            infection_rate: changed_params[3],
                                        }))
                                        .execute(conn)?;

                                    diesel::update(users::table)
                                        .filter(email.eq(user.email))
                                        .set(money.eq(money + reward))
                                        .execute(conn)?;

                                    diesel::update(status::table)
                                        .filter(id.eq(user_status_id))
                                        .set((current_event.eq(current_event + 1), postponed.eq(0)))
                                        .execute(conn)?;
                                    Ok(())
                                })?;

                                Ok(WSResponse::Event(ActionResponse {
                                    description: event_accept_message,
                                    is_success: true,
                                    simulation_data: SimulatorResponse {
                                        date: event.cur_date,
                                        region: data.region,
                                        payload,
                                        ideal_reproduction_number: changed_params[0],
                                        compliance_factor: changed_params[1],
                                        recovery_rate: changed_params[2],
                                        infection_rate: changed_params[3],
                                    },
                                }))
                            }
                            None => Ok(WSResponse::Error("Invalid request sent".to_string())),
                        }
                    }
                    EventAction::Decline => {
                        info!("Declined Event: {}", &event.id);
                        let event_decline_message =
                            &get_description(event.id.to_string(), user.curlevel);
                        let event_decline_message = match event_decline_message {
                            Read::ControlNews(_) => "Invalid Event".to_string(),
                            Read::EventNews(x) => x.reject.to_string(),
                            Read::Bs(_) => "Invalid Event".to_string(),
                        };
                        diesel::update(status)
                            .filter(id.eq(user_status_id))
                            .set((current_event.eq(current_event + 1), postponed.eq(0)))
                            .execute(conn)?;
                        Ok(WSResponse::Ok(event_decline_message))
                    }
                    EventAction::Postpone => {
                        info!("Postponed Event: {}", &event.id);
                        let event_postpone_message =
                            &get_description(event.id.to_string(), user.curlevel);
                        let event_postpone_message = match event_postpone_message {
                            Read::ControlNews(_) => "Invalid Event".to_string(),
                            Read::EventNews(x) => x.postpone.to_string(),
                            Read::Bs(_) => "Invalid Event".to_string(),
                        };
                        diesel::update(status)
                            .filter(id.eq(user_status_id))
                            .set(postponed.eq(postponed + 1))
                            .execute(conn)?;
                        Ok(WSResponse::Ok(event_postpone_message))
                    }
                }
            }
        }
    }
}

impl Save {
    #[instrument(skip(conn))]
    pub fn handle(
        payload: String,
        user: &extractors::Authenticated,
        conn: &PgConnection,
    ) -> Result<WSResponse, DbError> {
        use crate::db::schema::{regions, regions_status, status, users};
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

                let save_params = SimulatorParams {
                    susceptible: save_request.params.susceptible / POPULATION,
                    exposed: save_request.params.exposed / POPULATION,
                    infectious: save_request.params.infectious / POPULATION,
                    removed: save_request.params.removed / POPULATION,
                    current_reproduction_number: save_request.params.current_reproduction_number,
                    ideal_reproduction_number: save_request.params.ideal_reproduction_number,
                    compliance_factor: save_request.params.compliance_factor,
                    recovery_rate: save_request.params.recovery_rate,
                    infection_rate: save_request.params.infection_rate,
                };

                diesel::update(status::table)
                    .filter(status::id.eq(status_id))
                    .set(status::cur_date.eq(save_request.cur_date))
                    .execute(conn)?;

                diesel::update(regions::table)
                    .set(regions::simulation_params.eq(save_params))
                    .filter(regions::region_id.eq(save_request.region as i32))
                    .filter(
                        regions::id.eq_any(
                            regions_status::table
                                .filter(regions_status::status_id.eq(status_id))
                                .select(regions_status::region_id),
                        ),
                    )
                    .execute(conn)?;
                Ok(WSResponse::Info("Saving".to_string()))
            }
        }
    }
}
