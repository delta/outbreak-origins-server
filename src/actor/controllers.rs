use crate::actor::events::types::{
    ControlMeasure, ControlMeasureAction, ControlMeasureParams, Event, EventAction, EventParams,
    Seed, SimulatorParams, SimulatorResponse, Start, StartParams, WSResponse,
};
use crate::db::models;
use diesel::prelude::*;
use diesel::PgConnection;

use crate::actor::utils::serialize_state;
use crate::auth::extractors;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use virus_simulator::Simulator;

const POPULATION: f64 = 10000000.0;
const TOTAL_DAYS: f64 = 700.0;
const EVENT_POSTPONE_PENALTY: i32 = 100;

impl Seed {
    pub fn handle(user: &extractors::Authenticated, conn: &PgConnection) -> WSResponse {
        use crate::db::schema::users::dsl::*;
        let user = user.0.as_ref().unwrap();
        let user = users
            .filter(email.eq(user.email.clone()))
            .first::<models::User>(conn)
            .optional();

        let user = match user {
            Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
            Ok(x) => match x {
                None => return WSResponse::Error("User not found".to_string()),
                Some(y) => y,
            },
        };

        let file = format!("src/game/levels/{}/seed.json", user.curlevel);
        let path = Path::new(&file);
        let contents = fs::read_to_string(&path).expect("Something went wrong reading the file");
        WSResponse::Seed(contents)
    }
}

impl Start {
    pub fn handle(
        payload: String,
        user: &extractors::Authenticated,
        conn: &PgConnection,
    ) -> WSResponse {
        use crate::db::schema::status::dsl::*;
        use crate::db::schema::users;
        let user = user.0.as_ref().unwrap();
        let user = (users::table)
            .filter(users::email.eq(user.email.clone()))
            .first::<models::User>(conn)
            .optional();

        let mut user = match user {
            Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
            Ok(x) => match x {
                None => return WSResponse::Error("User not found".to_string()),
                Some(y) => y,
            },
        };

        // Get the associated status entry for this user
        let user_status_id: i32 = match user.status {
            Some(s_id) => s_id,
            None => {
                let s_id = match diesel::insert_into(status)
                    .default_values()
                    .get_result::<(i32, String, i32)>(conn)
                {
                    Ok(row) => row.0,
                    Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
                };

                match diesel::update((users::table).filter(users::email.eq(user.email)))
                    .set(users::status.eq(s_id))
                    .execute(conn)
                {
                    Ok(_) => (),
                    Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
                }

                s_id
            }
        };

        let region = serde_json::from_str::<Start>(&payload).unwrap().region;

        // Load the created regions for this user
        use crate::db::schema::regions;
        use crate::db::schema::regions_status;
        let existing_regions = match (regions_status::table)
            .filter(regions_status::status_id.eq(user_status_id))
            .select(regions_status::region_id)
            .load::<i32>(conn)
        {
            Ok(region_ids) => match (regions::table)
                .filter(regions::id.eq_any(region_ids))
                .select((regions::id, regions::region_id))
                .load::<(i32, i32)>(conn)
            {
                Ok(game_region_ids) => game_region_ids,
                Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
            },
            Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
        };

        // Get the region id
        let mut first_time = false;
        let user_region_id: i32 = match existing_regions.into_iter().find(|&x| x.1 == region) {
            Some(r_s_tuple) => r_s_tuple.0,
            None => {
                // Initialise the region
                let new_region_id = match diesel::insert_into(regions::table)
                    .values(regions::region_id.eq(&region))
                    .get_result::<(
                        i32,
                        i32,
                        SimulatorParams,
                        models::status::ActiveControlMeasures,
                    )>(conn)
                {
                    Ok(row) => row.0,
                    Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
                };

                // Create an entry in regions_status
                match diesel::insert_into(regions_status::table)
                    .values((
                        regions_status::status_id.eq(user_status_id),
                        regions_status::region_id.eq(new_region_id),
                    ))
                    .execute(conn)
                {
                    Ok(_) => (),
                    Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
                }

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
                    match diesel::update(regions::table.filter(regions::id.eq(user_region_id)))
                        .set(regions::simulation_params.eq(start_params))
                        .execute(conn)
                    {
                        Ok(_) => (),
                        Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
                    }

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
                    WSResponse::Start(SimulatorResponse {
                        region,
                        payload,
                        ideal_reproduction_number: start_params.ideal_reproduction_number,
                        compliance_factor: start_params.compliance_factor,
                        recovery_rate: start_params.recovery_rate,
                        infection_rate: start_params.infection_rate,
                    })
                }
                None => return WSResponse::Error("Internal Server Error".to_string()),
            }
        } else {
            match (regions::table)
                .filter(regions::id.eq(user_region_id))
                .select(regions::simulation_params)
                .first::<SimulatorParams>(conn)
            {
                Ok(sim_params) => {
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
                    WSResponse::Start(SimulatorResponse {
                        region,
                        payload,
                        ideal_reproduction_number: sim_params.ideal_reproduction_number,
                        compliance_factor: sim_params.compliance_factor,
                        recovery_rate: sim_params.recovery_rate,
                        infection_rate: sim_params.infection_rate,
                    })
                }
                Err(_) => WSResponse::Error("Internal Server Error".to_string()),
            }
        }
    }
}

impl ControlMeasure {
    pub fn handle(
        payload: String,
        user: &extractors::Authenticated,
        conn: &PgConnection,
    ) -> WSResponse {
        use crate::db::schema::{regions, regions_status, status, users};

        // User input wrapped in a Result
        let control_measure_request_result = serde_json::from_str::<ControlMeasure>(&payload);
        let user = user.0.as_ref().unwrap();
        let user = (users::table)
            .filter(users::email.eq(user.email.clone()))
            .first::<models::User>(conn)
            .optional();

        // Check if user present
        let user = match user {
            Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
            Ok(x) => match x {
                None => return WSResponse::Error("User not found".to_string()),
                Some(y) => y,
            },
        };

        let res = match control_measure_request_result {
            Err(_) => WSResponse::Error("Couldn't parse request".to_string()),
            // If valid request
            Ok(control_measure_request) => {
                // Reads data from control measure file
                let file = format!("src/game/levels/{}/control.json", user.curlevel);
                let path = Path::new(&file);
                let contents = match fs::read_to_string(&path) {
                    Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
                    Ok(val) => val,
                };

                let control_measure_data =
                    serde_json::from_str::<HashMap<String, ControlMeasureParams>>(&contents)
                        .unwrap();

                let zero_delta: &Vec<f64> = &vec![0_f64; 4];

                // Checks if the user has a status attached to them
                // If not creates one
                // Returns status id of user
                let status_id = match user.status {
                    Some(status_id) => status_id,
                    None => {
                        let s_id = match diesel::insert_into(status::table)
                            .default_values()
                            .get_result::<(i32, String, i32)>(conn)
                        {
                            Ok(r) => r.0,
                            Err(_) => {
                                return WSResponse::Error("Internal Server Error".to_string())
                            }
                        };
                        if diesel::update(
                            (users::table).filter(users::email.eq(user.email.clone())),
                        )
                        .set(users::status.eq(s_id))
                        .execute(conn)
                        .is_err()
                        {
                            return WSResponse::Error("Internal Server Error".to_string());
                        }
                        s_id
                    }
                };

                let (mut active_control_measures, existing_delta) = match (regions::table)
                    .inner_join(regions_status::table)
                    .filter(regions_status::status_id.eq(status_id))
                    .filter(regions::region_id.eq(control_measure_request.region as i32))
                    .select(regions::active_control_measures)
                    .load::<models::status::ActiveControlMeasures>(conn)
                {
                    Ok(x) => {
                        if x.is_empty() {
                            let regions_row = match diesel::insert_into(regions::table)
                                .values(
                                    regions::region_id.eq(control_measure_request.region as i32),
                                )
                                .get_result::<(
                                    i32,
                                    i32,
                                    SimulatorParams,
                                    models::status::ActiveControlMeasures,
                                )>(conn)
                            {
                                Ok(val) => val,
                                Err(_) => {
                                    return WSResponse::Error("Internal Server Error".to_string())
                                }
                            };

                            if diesel::insert_into(regions_status::table)
                                .values((
                                    regions_status::status_id.eq(status_id),
                                    regions_status::region_id.eq(regions_row.0),
                                ))
                                .execute(conn)
                                .is_err()
                            {
                                return WSResponse::Error("Internal Server Error".to_string());
                            }
                            (regions_row.3 .0, zero_delta)
                        } else {
                            match x[0].0.get(&control_measure_request.name) {
                                Some(val) => {
                                    match control_measure_data
                                        .get(&control_measure_request.name)
                                        .unwrap()
                                        .levels
                                        .get(val)
                                    {
                                        Some(y) => (x[0].0.clone(), &y.params_delta),
                                        // Should never happen since the user can only have
                                        // a control measure if present in our file
                                        None => {
                                            return WSResponse::Error(
                                                "Internal Server Error".to_string(),
                                            )
                                        }
                                    }
                                }
                                None => (x[0].0.clone(), zero_delta),
                            }
                        }
                    }
                    Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
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
                                return WSResponse::Error("Not enough money".to_string());
                            }
                            if control_measure_request.action == ControlMeasureAction::Remove
                                && existing_delta == zero_delta
                            {
                                return WSResponse::Error(
                                    "Control measure was not applied".to_string(),
                                );
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
                                    // active_control_measures.0.
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

                            let f =
                                sim.simulate(0_f64, TOTAL_DAYS - control_measure_request.cur_date);
                            let payload = serialize_state(&f, POPULATION);

                            let res = conn.transaction::<_, diesel::result::Error, _>(|| {
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
                            });
                            match res {
                                Ok(_) => WSResponse::Control(SimulatorResponse {
                                    region: control_measure_request.region as i32,
                                    payload,
                                    ideal_reproduction_number: changed_params[0],
                                    compliance_factor: changed_params[1],
                                    recovery_rate: changed_params[2],
                                    infection_rate: changed_params[3],
                                }),
                                Err(_) => WSResponse::Error("Internal Server Error".to_string()),
                            }
                        }
                        None => WSResponse::Error("Level not found".to_string()),
                    },
                    None => WSResponse::Error("No control measure found".to_string()),
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
    ) -> WSResponse {
        use crate::db::schema::users::dsl::*;
        let control_measure = serde_json::from_str::<ControlMeasure>(&payload);
        let user = user.0.as_ref().unwrap();
        let user = users
            .filter(email.eq(user.email.clone()))
            .first::<models::User>(conn)
            .optional();
        let mut user = match user {
            Err(_) => return WSResponse::Error("Internal Server Error".to_string()),
            Ok(x) => match x {
                None => return WSResponse::Error("User not found".to_string()),
                Some(y) => y,
            },
        };

        let user_status_id = match user.status {
            Some(x) => x,
            None => return WSResponse::Error("Internal Server Error".to_string()),
        };

        let event = serde_json::from_str::<Event>(&payload);

        match event {
            Err(_) => return WSResponse::Error("Couldn't parse request".to_string()),

            Ok(event) => {
                let file = format!("src/game/levels/{}/event.json", user.curlevel);
                let path = Path::new(&file);
                let contents =
                    fs::read_to_string(&path).expect("Something  went wrong reading the file");
                let event_data =
                    serde_json::from_str::<HashMap<String, EventParams>>(&contents).unwrap();

                use crate::db::schema::status::dsl::*;

                match event.action {
                    EventAction::Request => match event_data.get(&event.id.to_string()) {
                        Some(data) => {
                            match status
                                .filter(id.eq(user_status_id))
                                .first::<(i32, String, i32)>(conn)
                            {
                                Err(_) => {
                                    return WSResponse::Error("Internal Server Error".to_string())
                                }
                                Ok(user_status) => {
                                    match diesel::update(status)
                                        .filter(id.eq(user_status_id))
                                        .set((current_event.eq(data.name), postponed.eq(0)))
                                        .execute(conn)
                                    {
                                        Err(_) => {
                                            return WSResponse::Error(
                                                "Internal Server Error".to_string(),
                                            )
                                        }
                                        Ok(_) => (),
                                    }
                                }
                            }
                            return WSResponse::Seed(
                                serde_json::to_string::<EventParams>(&data).unwrap(),
                            );
                        }
                        None => return WSResponse::Error("Couldn't read the file".to_string()),
                    },
                    EventAction::Accept => match event_data.get(&event.id.to_string()) {
                        Some(data) => {
                            let reward =
                                match status
                                    .filter(id.eq(user_status_id))
                                    .first::<(i32, String, i32)>(conn)
                                {
                                    Err(_) => {
                                        return WSResponse::Error(
                                            "Internal Server Error".to_string(),
                                        )
                                    }
                                    Ok(user_status) => {
                                        if user_status.1 != data.name {
                                            return WSResponse::Error(
                                                "Cannot Accept event which wasn't requested"
                                                    .to_string(),
                                            );
                                        }
                                        data.reward - user_status.2 * EVENT_POSTPONE_PENALTY
                                    }
                                };

                            match diesel::update(users.filter(email.eq(user.email)))
                                .set(money.eq(money + reward))
                                .execute(conn)
                            {
                                Err(_) => {
                                    return WSResponse::Error("Internal Server Error".to_string())
                                }
                                Ok(_) => (),
                            }

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

                            let f = sim.simulate(0_f64, TOTAL_DAYS - event.cur_date);
                            let payload = serialize_state(&f, POPULATION);

                            return WSResponse::Event(SimulatorResponse {
                                region: data.region,
                                payload,
                                ideal_reproduction_number: changed_params[0],
                                compliance_factor: changed_params[1],
                                recovery_rate: changed_params[2],
                                infection_rate: changed_params[3],
                            });
                        }
                        None => return WSResponse::Error("Invalid request sent".to_string()),
                    },
                    EventAction::Decline => {
                        match status
                            .filter(id.eq(user_status_id))
                            .first::<(i32, String, i32)>(conn)
                        {
                            Err(_) => {
                                return WSResponse::Error("Internal Server Error".to_string())
                            }
                            Ok(user_status) => {
                                match diesel::update(status)
                                    .filter(id.eq(user_status_id))
                                    .set((current_event.eq("None"), postponed.eq(0)))
                                    .execute(conn)
                                {
                                    Err(_) => {
                                        return WSResponse::Error(
                                            "Internal Server Error".to_string(),
                                        )
                                    }
                                    Ok(_) => return WSResponse::Ok("Declined".to_string()),
                                }
                            }
                        }
                    }
                    EventAction::Postpone => {
                        match status
                            .filter(id.eq(user_status_id))
                            .first::<(i32, String, i32)>(conn)
                        {
                            Err(_) => {
                                return WSResponse::Error("Internal Server Error".to_string())
                            }
                            Ok(user_status) => {
                                match diesel::update(status)
                                    .filter(id.eq(user_status_id))
                                    .set(postponed.eq(postponed + 1))
                                    .execute(conn)
                                {
                                    Err(_) => {
                                        return WSResponse::Error(
                                            "Internal Server Error".to_string(),
                                        )
                                    }
                                    Ok(_) => return WSResponse::Ok("Postponed".to_string()),
                                }
                            }
                        }
                    }
                }
            }
        };
    }
}
