CREATE TABLE IF NOT EXISTS rooms (
    room_id   TEXT             PRIMARY KEY,
    unit_size DOUBLE PRECISION NOT NULL,
    unit_goal DOUBLE PRECISION NOT NULL
);

CREATE TABLE IF NOT EXISTS players (
    id        SERIAL           PRIMARY KEY,
    room_id   TEXT             NOT NULL REFERENCES rooms(room_id),
    username  TEXT             NOT NULL,
    score     DOUBLE PRECISION NOT NULL DEFAULT 0.0,

    CONSTRAINT unique_player_in_room UNIQUE (room_id, username)
);

CREATE INDEX IF NOT EXISTS idx_players_room_id       ON players (room_id);
CREATE INDEX IF NOT EXISTS idx_players_room_username ON players (room_id, username);
