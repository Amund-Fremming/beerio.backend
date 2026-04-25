# Beerio Backend — API Reference

Base URL: `http://localhost:3000`

---

## Health

### `GET /health`

Returns a simple liveness check.

**Response `200 OK`**

```json
"ok"
```

---

### `GET /health/detailed`

Returns detailed health including database connectivity.

**Response `200 OK`**

```json
{
  "server": "ok",
  "database": "ok"
}
```

**Response `503 Service Unavailable`** (if DB is unreachable)

```json
{
  "server": "ok",
  "database": "error"
}
```

---

## Rooms

### `POST /rooms`

Create a new room.

**Request Body**

```json
{
  "unit_size": 0.33, // float — unit size in litres, 0.33 or 0.5
  "unit_goal": 10.0 // float — target number of units for the game
}
```

**Response `201 Created`**

```json
{
  "room_id": "abc123" // string — unique room identifier
}
```

**Errors**
| Status | Reason |
|--------|--------|
| `400` | Invalid `unit_size` (must be 0.33 or 0.5) |

---

### `GET /rooms/:room_id`

Get the current state of a room.

**Path Params**
| Param | Type | Description |
|-------|------|-------------|
| `room_id` | string | The unique room ID |

**Response `200 OK`**

```json
{
  "room_id": "abc123",
  "unit_size": 0.33,
  "unit_goal": 10.0,
  "players": [
    {
      "username": "Alice",
      "score": 2.66 // float — total units consumed
    },
    {
      "username": "Bob",
      "score": 1.5
    }
  ]
}
```

**Errors**
| Status | Reason |
|--------|--------|
| `404` | Room not found |

---

### `POST /rooms/:room_id/join`

Add a player to a room.

**Path Params**
| Param | Type | Description |
|-------|------|-------------|
| `room_id` | string | The unique room ID |

**Request Body**

```json
{
  "username": "Alice" // string — must be unique within the room
}
```

**Response `201 Created`**

```json
{
  "username": "Alice",
  "score": 0.0
}
```

**Errors**
| Status | Reason |
|--------|--------|
| `400` | Username already taken in this room |
| `404` | Room not found |

---

### `POST /rooms/:room_id/players/:username/drink`

Record a drink (enhet) for a player.

**Path Params**
| Param | Type | Description |
|-------|------|-------------|
| `room_id` | string | The unique room ID |
| `username` | string | The player's username |

**Request Body**

```json
{
  "unit_size": 0.5 // float — size of the drink added (0.33 or 0.5)
}
```

> The score is stored in "units" relative to the room's `unit_size`.  
> Example: room `unit_size` = 0.33, drink added = 0.5 → score increases by `0.5 / 0.33 ≈ 1.515`

**Response `200 OK`**

```json
{
  "username": "Alice",
  "score": 3.515 // updated total score in units
}
```

**Errors**
| Status | Reason |
|--------|--------|
| `400` | Invalid `unit_size` |
| `404` | Room or player not found |

---

## Database Schema

```sql
CREATE TABLE rooms (
    room_id    TEXT        PRIMARY KEY,
    unit_size  DOUBLE PRECISION NOT NULL,  -- 0.33 or 0.5
    unit_goal  DOUBLE PRECISION NOT NULL
);

CREATE TABLE players (
    id         SERIAL      PRIMARY KEY,
    room_id    TEXT        NOT NULL REFERENCES rooms(room_id),
    username   TEXT        NOT NULL,
    score      DOUBLE PRECISION NOT NULL DEFAULT 0.0,

    CONSTRAINT unique_player_in_room UNIQUE (room_id, username)
);

CREATE INDEX idx_players_room_id ON players (room_id);
CREATE INDEX idx_players_room_username ON players (room_id, username);
```

---

## DTO Summary

| DTO                  | Used in                                        | Fields                                                        |
| -------------------- | ---------------------------------------------- | ------------------------------------------------------------- |
| `CreateRoomRequest`  | `POST /rooms` body                             | `unit_size: f64`, `unit_goal: f64`                            |
| `CreateRoomResponse` | `POST /rooms` response                         | `room_id: String`                                             |
| `RoomState`          | `GET /rooms/:id` response                      | `room_id`, `unit_size`, `unit_goal`, `players: PlayerScore[]` |
| `PlayerScore`        | nested in `RoomState`                          | `username: String`, `score: f64`                              |
| `JoinRoomRequest`    | `POST /rooms/:id/join` body                    | `username: String`                                            |
| `DrinkRequest`       | `POST /rooms/:id/players/:username/drink` body | `unit_size: f64`                                              |
| `DetailedHealth`     | `GET /health/detailed` response                | `server: String`, `database: String`                          |
