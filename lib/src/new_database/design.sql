--- SECTION: Config

-- Already integrated

--- SECTION: local music files

-- Already integrated

--- SECTION: podcasts

-- the table for all top-level podcasts
CREATE TABLE IF NOT EXISTS podcasts(
    id INTEGER PRIMARY KEY,
    -- as podcasts are not just local, a url is required
    url TEXT NOT NULL UNIQUE,
    -- refuse podcasts which do not have a title
    title TEXT NOT NULL,
    -- the author could not be present and is not necessary
    author_display TEXT,
    explicit BOOLEAN,
    -- optional image url
    image_url TEXT,
    -- the time the podcast was last checked for updates
    last_checked DATE NOT NULL,
    -- optional description of the podcast
    description TEXT
);

-- relation table for a podcast's artist / author
-- this is so that podcasts with "ArtistA feat. ArtistB" can be searched for either
-- entry will get deleted if either the artist or the podcast get dropped
CREATE TABLE IF NOT EXISTS podcasts_artist(
    id INTEGER PRIMARY KEY,
    podcast INTEGER NOT NULL REFERENCES podcasts(id) ON DELETE CASCADE,
    artist INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE
);

-- the table for episodes of a podcast
-- NOTE: local episode files should be named with the id of "podcast_episodes" and be auto-discovered
CREATE TABLE IF NOT EXISTS podcast_episodes(
    id INTEGER PRIMARY KEY,
    -- the podcast this episode is for, delete on podcast delete
    podcast INTEGER NOT NULL REFERENCES podcasts(id) ON DELETE CASCADE,
    -- preferably this should always be present, but not all feeds may have it
    guid TEXT,
    -- the title is required, if not present the app should auto-fill with something
    title TEXT NOT NULL,
    -- the url to the individual file for direct play
    url TEXT NOT NULL,
    -- indicator for if the episode had already been fully played
    played BOOLEAN NOT NULL,
    -- indicator for if the episode should be hidden
    hidden BOOLEAN NOT NULL,
    -- last known played position, if NULL should assume 0
    last_position INTEGER,
    -- optional duration, preferably should always be present
    duration INTEGER,
    -- optional description of the episode
    description TEXT
);
