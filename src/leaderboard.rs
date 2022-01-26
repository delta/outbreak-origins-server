use crate::db::models;
use crate::db::types::DbError;

use crate::PgPool;
use actix_web::{get, web, Error, HttpResponse};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

fn get_leaderboard(conn: &PgConnection) -> Result<Vec<models::LeaderboardEntry>, DbError> {
    use crate::db::schema::users::dsl::*;
    let users_result: QueryResult<Vec<models::User>> = users.load(conn);
    let _my_users = users_result.expect("Error loading Users");

    let leaderboard_data = users
        .select((firstname, lastname, score))
        .order(score.desc())
        .load::<models::LeaderboardEntry>(conn)
        .expect("Error loading leaderboard");

    Ok(leaderboard_data)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LeaderboardResponse {
    pub status: String,
    pub data: Vec<models::LeaderboardEntry>,
}

#[get("/leaderboard")]
pub async fn leaderboard(pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    let leaderboard = web::block(move || {
        let conn = pool.get()?;
        get_leaderboard(&conn)
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().json(LeaderboardResponse {
            status: String::from("Failed"),
            data: vec![],
        })
    })?;
    Ok(HttpResponse::Ok().json(LeaderboardResponse {
        status: String::from("Success"),
        data: leaderboard,
    }))
}

#[cfg(test)]
mod tests {
    use test_manager::TestDbManager;
    use super::*;
    use crate::tests::utils::insert_test_user;
    use actix_web::{test, App};

    fn leaderboard_routes(cfg: &mut web::ServiceConfig) {
        cfg.service(web::scope("/").service(leaderboard));
    }

    #[actix_rt::test]
    async fn test_get_leaderboard() {
        let test_db = TestDbManager::new();
        let mut test_app = test::init_service(
            App::new()
                .data(test_db.conn_pool.clone())
                .configure(leaderboard_routes),
        )
        .await;

        let test_db_conn = test_db
            .conn_pool
            .get()
            .expect("Couldn't get test DB connection");

        let test_users = vec![
            models::NewUser {
                firstname: "User".to_string(),
                lastname: "A".to_string(),
                password: "".to_string(),
                email: "usera@email.com".to_string(),
                score: 10,
            },
            models::NewUser {
                firstname: "User".to_string(),
                lastname: "B".to_string(),
                password: "".to_string(),
                email: "userb@email.com".to_string(),
                score: 50,
            },
            models::NewUser {
                firstname: "User".to_string(),
                lastname: "C".to_string(),
                password: "".to_string(),
                email: "userc@email.com".to_string(),
                score: 0,
            },
        ];

        for user in test_users.iter() {
            let res = insert_test_user(&user, &test_db_conn);
            res.expect("Couldn't insert test user");
        }

        let req = test::TestRequest::get().uri("/leaderboard").to_request();

        let res: LeaderboardResponse = test::read_response_json(&mut test_app, req).await;

        let mut scores = Vec::new();
        for entry in res.data.iter() {
            scores.push(entry.score);
        }

        assert_eq!(scores, vec![50, 10, 0]);
    }
}
