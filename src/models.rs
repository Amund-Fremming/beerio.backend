use serde::{Deserialize, Serialize};

// --- Rooms ---

#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub unit_size: f64,
    pub unit_goal: f64,
}

#[derive(Debug, Serialize)]
pub struct CreateRoomResponse {
    pub room_id: String,
}

#[derive(Debug, Serialize)]
pub struct RoomState {
    pub room_id: String,
    pub unit_size: f64,
    pub unit_goal: f64,
    pub players: Vec<PlayerScore>,
}

// --- Players ---

#[derive(Debug, Deserialize)]
pub struct JoinRoomRequest {
    pub username: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PlayerScore {
    pub username: String,
    pub score: f64,
}

// --- Drinks ---

#[derive(Debug, Deserialize)]
pub struct DrinkRequest {
    pub unit_size: f64,
}

// --- Health ---

#[derive(Debug, Serialize)]
pub struct DetailedHealth {
    pub server: String,
    pub database: String,
}

// --- DB row types ---

#[derive(Debug, sqlx::FromRow)]
pub struct RoomRow {
    pub room_id: String,
    pub unit_size: f64,
    pub unit_goal: f64,
}
