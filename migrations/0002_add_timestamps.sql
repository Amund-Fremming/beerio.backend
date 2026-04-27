-- Add updated_at to both tables
ALTER TABLE rooms   ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
ALTER TABLE players ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Generic function to refresh updated_at on row updates
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger: keep rooms.updated_at current on direct updates
CREATE TRIGGER rooms_updated_at
    BEFORE UPDATE ON rooms
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Trigger: keep players.updated_at current on direct updates
CREATE TRIGGER players_updated_at
    BEFORE UPDATE ON players
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Propagate activity up to the parent room whenever a player row is
-- inserted (join) or updated (drink / undo drink), so the room's
-- updated_at reflects the last time *anything* happened in that room.
CREATE OR REPLACE FUNCTION touch_room_on_player_change()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE rooms SET updated_at = NOW() WHERE room_id = NEW.room_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER players_touch_room
    AFTER INSERT OR UPDATE ON players
    FOR EACH ROW EXECUTE FUNCTION touch_room_on_player_change();

-- Add ON DELETE CASCADE so deleting a room automatically removes its players
ALTER TABLE players DROP CONSTRAINT players_room_id_fkey;
ALTER TABLE players ADD CONSTRAINT players_room_id_fkey
    FOREIGN KEY (room_id) REFERENCES rooms(room_id) ON DELETE CASCADE;
