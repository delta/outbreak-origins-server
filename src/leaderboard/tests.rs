#[cfg(test)]
use crate::leaderboard::routes;

#[actix_rt::test]
#[cfg(test)]
async fn test_get_leaderboard() {
    use crate::db::models;
    use crate::leaderboard::response::LeaderboardResponse;
    use crate::tests::utils::insert_test_user;
    use actix_web::{test, App};
    use test_manager::TestDbManager;

    let test_db = TestDbManager::new();
    let mut test_app = test::init_service(
        App::new()
            .data(test_db.conn_pool.clone())
            .configure(routes::leaderboard_routes),
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
            money: 100,
        },
        models::NewUser {
            firstname: "User".to_string(),
            lastname: "B".to_string(),
            password: "".to_string(),
            email: "userb@email.com".to_string(),
            score: 50,
            money: 25,
        },
        models::NewUser {
            firstname: "User".to_string(),
            lastname: "C".to_string(),
            password: "".to_string(),
            email: "userc@email.com".to_string(),
            score: 0,
            money: 0,
        },
    ];

    for user in test_users.iter() {
        let res = insert_test_user(&user, &test_db_conn);
        res.expect("Couldn't insert test user");
    }

    let req = test::TestRequest::get().uri("/leaderboard/1").to_request();

    let res: LeaderboardResponse = test::read_response_json(&mut test_app, req).await;

    let mut scores = Vec::new();
    for entry in res.data.iter() {
        scores.push(entry.score);
    }

    assert_eq!(scores, vec![50, 10, 0]);
}
