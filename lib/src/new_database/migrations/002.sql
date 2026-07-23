--- SECTION: Sort support (most played, recency, first added, frecency)

-- Total number of times the track has been started (incremented once per start).
ALTER TABLE tracks ADD COLUMN total_play_count INTEGER NOT NULL DEFAULT 0;
-- Unix epoch seconds of the last time the track was started. NULL if never played.
ALTER TABLE tracks ADD COLUMN last_played_at INTEGER;

-- added_at was originally defined as DATE (RFC 3339 strings) but we now need
-- INTEGER (unix epoch seconds) for numeric sorting. SQLite does not support
-- altering a column type in place, so drop and re-create it.
ALTER TABLE tracks ADD COLUMN added_at_new INTEGER;
-- Preserve existing values by converting the old DATE string to unix epoch.
UPDATE tracks SET added_at_new = CAST(strftime('%s', added_at) AS INTEGER);
ALTER TABLE tracks DROP COLUMN added_at;
ALTER TABLE tracks RENAME COLUMN added_at_new TO added_at;
