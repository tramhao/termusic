CREATE TABLE IF NOT EXISTS tracks(
    id INTEGER PRIMARY KEY,
    artist TEXT,
    title TEXT,
    album TEXT,
    genre TEXT,
    file TEXT NOT NULL,
    duration INTERGER,
    name TEXT,
    ext TEXT,
    directory TEXT,
    last_modified TEXT,
    last_position INTERGER
);
