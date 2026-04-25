use axum_test::TestServer;
use serde_json::json;
use sqlx::PgPool;

use crate::{db::create_pool, handlers::router};

async fn test_server() -> (TestServer, PgPool) {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = create_pool(&url).await;

    // Clean state
    sqlx::query("DELETE FROM players")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM rooms")
        .execute(&pool)
        .await
        .unwrap();

    let app = router().with_state(pool.clone());
    let server = TestServer::new(app).unwrap();
    (server, pool)
}

#[tokio::test]
async fn test_health() {
    let (server, _) = test_server().await;
    let res = server.get("/health").await;
    res.assert_status_ok();
}

#[tokio::test]
async fn test_health_detailed() {
    let (server, _) = test_server().await;
    let res = server.get("/health/detailed").await;
    res.assert_status_ok();
    let body = res.json::<serde_json::Value>();
    assert_eq!(body["server"], "ok");
    assert_eq!(body["database"], "ok");
}

#[tokio::test]
async fn test_create_and_get_room() {
    let (server, _) = test_server().await;

    let res = server
        .post("/rooms")
        .json(&json!({"unit_size": 0.33, "unit_goal": 10.0}))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let body = res.json::<serde_json::Value>();
    let room_id = body["room_id"].as_str().unwrap().to_string();

    let res = server.get(&format!("/rooms/{room_id}")).await;
    res.assert_status_ok();
    let room = res.json::<serde_json::Value>();
    assert_eq!(room["room_id"], room_id);
    assert_eq!(room["unit_size"], 0.33);
    assert_eq!(room["unit_goal"], 10.0);
    assert!(room["players"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_room_invalid_unit_size() {
    let (server, _) = test_server().await;
    let res = server
        .post("/rooms")
        .json(&json!({"unit_size": 0.25, "unit_goal": 5.0}))
        .await;
    res.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_join_room() {
    let (server, _) = test_server().await;

    let res = server
        .post("/rooms")
        .json(&json!({"unit_size": 0.5, "unit_goal": 5.0}))
        .await;
    let room_id = res.json::<serde_json::Value>()["room_id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = server
        .post(&format!("/rooms/{room_id}/join"))
        .json(&json!({"username": "Alice"}))
        .await;
    res.assert_status(axum::http::StatusCode::CREATED);
    let player = res.json::<serde_json::Value>();
    assert_eq!(player["username"], "Alice");
    assert_eq!(player["score"], 0.0);
}

#[tokio::test]
async fn test_join_room_duplicate_username() {
    let (server, _) = test_server().await;

    let res = server
        .post("/rooms")
        .json(&json!({"unit_size": 0.5, "unit_goal": 5.0}))
        .await;
    let room_id = res.json::<serde_json::Value>()["room_id"]
        .as_str()
        .unwrap()
        .to_string();

    server
        .post(&format!("/rooms/{room_id}/join"))
        .json(&json!({"username": "Bob"}))
        .await;

    let res = server
        .post(&format!("/rooms/{room_id}/join"))
        .json(&json!({"username": "Bob"}))
        .await;
    res.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_drink_updates_score() {
    let (server, _) = test_server().await;

    let res = server
        .post("/rooms")
        .json(&json!({"unit_size": 0.33, "unit_goal": 10.0}))
        .await;
    let room_id = res.json::<serde_json::Value>()["room_id"]
        .as_str()
        .unwrap()
        .to_string();

    server
        .post(&format!("/rooms/{room_id}/join"))
        .json(&json!({"username": "Charlie"}))
        .await;

    // Add a 0.5 drink to a 0.33-unit room → delta = 0.5/0.33 ≈ 1.515
    let res = server
        .post(&format!("/rooms/{room_id}/players/Charlie/drink"))
        .json(&json!({"unit_size": 0.5}))
        .await;
    res.assert_status_ok();
    let player = res.json::<serde_json::Value>();
    assert_eq!(player["username"], "Charlie");
    let score = player["score"].as_f64().unwrap();
    assert!((score - (0.5f64 / 0.33f64)).abs() < 0.001);
}

#[tokio::test]
async fn test_get_room_not_found() {
    let (server, _) = test_server().await;
    let res = server.get("/rooms/doesnotexist").await;
    res.assert_status(axum::http::StatusCode::NOT_FOUND);
}
