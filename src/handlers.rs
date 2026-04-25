use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateRoomRequest, CreateRoomResponse, DetailedHealth, DrinkRequest, JoinRoomRequest,
    PlayerScore, RoomRow, RoomState,
};

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/health", get(health))
        .route("/health/detailed", get(health_detailed))
        .route("/rooms", post(create_room))
        .route("/rooms/:room_id", get(get_room))
        .route("/rooms/:room_id/join", post(join_room))
        .route(
            "/rooms/:room_id/players/:username/drink",
            post(add_drink).delete(undo_drink),
        )
}

// GET /health
async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json("ok"))
}

// GET /health/detailed
async fn health_detailed(State(pool): State<PgPool>) -> impl IntoResponse {
    let db_status = sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map(|_| "ok")
        .unwrap_or("error");

    let status = if db_status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(DetailedHealth {
            server: "ok".into(),
            database: db_status.into(),
        }),
    )
}

// POST /rooms
async fn create_room(
    State(pool): State<PgPool>,
    Json(body): Json<CreateRoomRequest>,
) -> impl IntoResponse {
    if body.unit_size != 0.33 && body.unit_size != 0.5 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "unit_size must be 0.33 or 0.5"})),
        );
    }

    let room_id = Uuid::new_v4().to_string()[..4].to_uppercase();

    sqlx::query("INSERT INTO rooms (room_id, unit_size, unit_goal) VALUES ($1, $2, $3)")
        .bind(&room_id)
        .bind(body.unit_size)
        .bind(body.unit_goal)
        .execute(&pool)
        .await
        .unwrap();

    (
        StatusCode::CREATED,
        Json(serde_json::json!(CreateRoomResponse { room_id })),
    )
}

// GET /rooms/:room_id
async fn get_room(State(pool): State<PgPool>, Path(room_id): Path<String>) -> impl IntoResponse {
    let room = sqlx::query_as::<_, RoomRow>(
        "SELECT room_id, unit_size, unit_goal FROM rooms WHERE room_id = $1",
    )
    .bind(&room_id)
    .fetch_optional(&pool)
    .await
    .unwrap();

    let Some(room) = room else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "room not found"})),
        );
    };

    let players = sqlx::query_as::<_, PlayerScore>(
        "SELECT username, score FROM players WHERE room_id = $1 ORDER BY score DESC",
    )
    .bind(&room_id)
    .fetch_all(&pool)
    .await
    .unwrap();

    (
        StatusCode::OK,
        Json(serde_json::json!(RoomState {
            room_id: room.room_id,
            unit_size: room.unit_size,
            unit_goal: room.unit_goal,
            players,
        })),
    )
}

// POST /rooms/:room_id/join
async fn join_room(
    State(pool): State<PgPool>,
    Path(room_id): Path<String>,
    Json(body): Json<JoinRoomRequest>,
) -> impl IntoResponse {
    let room_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM rooms WHERE room_id = $1)")
            .bind(&room_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    if !room_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "room not found"})),
        );
    }

    let existing = sqlx::query_as::<_, PlayerScore>(
        "SELECT username, score FROM players WHERE room_id = $1 AND username = $2",
    )
    .bind(&room_id)
    .bind(&body.username)
    .fetch_optional(&pool)
    .await
    .unwrap();

    if let Some(player) = existing {
        return (StatusCode::OK, Json(serde_json::json!(player)));
    }

    sqlx::query("INSERT INTO players (room_id, username, score) VALUES ($1, $2, 0.0)")
        .bind(&room_id)
        .bind(&body.username)
        .execute(&pool)
        .await
        .unwrap();

    (
        StatusCode::CREATED,
        Json(serde_json::json!(PlayerScore {
            username: body.username,
            score: 0.0,
        })),
    )
}

// POST /rooms/:room_id/players/:username/drink
async fn add_drink(
    State(pool): State<PgPool>,
    Path((room_id, username)): Path<(String, String)>,
    Json(body): Json<DrinkRequest>,
) -> impl IntoResponse {
    if body.unit_size != 0.33 && body.unit_size != 0.5 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "unit_size must be 0.33 or 0.5"})),
        );
    }

    let room = sqlx::query_as::<_, RoomRow>(
        "SELECT room_id, unit_size, unit_goal FROM rooms WHERE room_id = $1",
    )
    .bind(&room_id)
    .fetch_optional(&pool)
    .await
    .unwrap();

    let Some(room) = room else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "room not found"})),
        );
    };

    let delta = body.unit_size / room.unit_size;

    let rows = sqlx::query_scalar::<_, i64>(
        "UPDATE players SET score = score + $1 WHERE room_id = $2 AND username = $3
         RETURNING 1",
    )
    .bind(delta)
    .bind(&room_id)
    .bind(&username)
    .fetch_optional(&pool)
    .await
    .unwrap();

    if rows.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "player not found"})),
        );
    }

    let players = sqlx::query_as::<_, PlayerScore>(
        "SELECT username, score FROM players WHERE room_id = $1 ORDER BY score DESC",
    )
    .bind(&room_id)
    .fetch_all(&pool)
    .await
    .unwrap();

    (
        StatusCode::OK,
        Json(serde_json::json!(RoomState {
            room_id: room.room_id,
            unit_size: room.unit_size,
            unit_goal: room.unit_goal,
            players,
        })),
    )
}

// DELETE /rooms/:room_id/players/:username/drink
async fn undo_drink(
    State(pool): State<PgPool>,
    Path((room_id, username)): Path<(String, String)>,
    Json(body): Json<DrinkRequest>,
) -> impl IntoResponse {
    if body.unit_size != 0.33 && body.unit_size != 0.5 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "unit_size must be 0.33 or 0.5"})),
        );
    }

    let room = sqlx::query_as::<_, RoomRow>(
        "SELECT room_id, unit_size, unit_goal FROM rooms WHERE room_id = $1",
    )
    .bind(&room_id)
    .fetch_optional(&pool)
    .await
    .unwrap();

    let Some(room) = room else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "room not found"})),
        );
    };

    let delta = body.unit_size / room.unit_size;

    let rows = sqlx::query_scalar::<_, i64>(
        "UPDATE players SET score = GREATEST(score - $1, 0) WHERE room_id = $2 AND username = $3
         RETURNING 1",
    )
    .bind(delta)
    .bind(&room_id)
    .bind(&username)
    .fetch_optional(&pool)
    .await
    .unwrap();

    if rows.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "player not found"})),
        );
    }

    let players = sqlx::query_as::<_, PlayerScore>(
        "SELECT username, score FROM players WHERE room_id = $1 ORDER BY score DESC",
    )
    .bind(&room_id)
    .fetch_all(&pool)
    .await
    .unwrap();

    (
        StatusCode::OK,
        Json(serde_json::json!(RoomState {
            room_id: room.room_id,
            unit_size: room.unit_size,
            unit_goal: room.unit_goal,
            players,
        })),
    )
}
