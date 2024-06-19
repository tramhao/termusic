CREATE TABLE IF NOT EXISTS podcasts (
    id INTEGER PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    url TEXT NOT NULL UNIQUE,
    description TEXT,
    author TEXT,
    explicit INTEGER,
    image_url TEXT,
    last_checked INTEGER
);

CREATE TABLE IF NOT EXISTS episodes (
    id INTEGER PRIMARY KEY NOT NULL,
    podcast_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    guid TEXT,
    description TEXT,
    pubdate INTEGER,
    duration INTEGER,
    played INTEGER,
    hidden INTEGER,
    last_position INTERGER,
    image_url TEXT,
    FOREIGN KEY(podcast_id) REFERENCES podcasts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY NOT NULL,
    episode_id INTEGER NOT NULL,
    path TEXT NOT NULL UNIQUE,
    FOREIGN KEY (episode_id) REFERENCES episodes(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS version (
    id INTEGER PRIMARY KEY NOT NULL,
    version TEXT NOT NULL
);
