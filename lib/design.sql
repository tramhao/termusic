--- SECTION: Config

-- table for extra information in a key-value style
-- examples ("key: val_format"): "db_created_at: DATE", "last_used_version: TERMUSIC_VERSION"
CREATE TABLE IF NOT EXISTS config(
    key TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
)

--- SECTION: local music files

-- the table for all local music files to use as reference
CREATE TABLE IF NOT EXISTS tracks(
    id INTEGER PRIMARY KEY,
    -- the file's directory
    file_dir TEXT NOT NULL,
    -- the file's name
    file_stem TEXT NOT NULL,
    -- the file's extension
    file_ext TEXT NOT NULL,
    -- duration parsing may fail or somehow not be available
    duration INTEGER,
    -- last known played position, if NULL should assume 0
    last_position INTEGER,
    -- the date the file was first added to the database
    added_at DATE,
    -- the album the file belongs to, if any
    -- set "NULL" on delete as deleting a album does not inheritly mean the file is deleted
    album INTEGER REFERENCES albums(id) ON DELETE SET NULL
);

-- unique index on the tracks.file_* columns as those combine to be one path
CREATE UNIQUE INDEX IF NOT EXISTS tracks_files ON tracks(file_dir, file_stem, file_ext);

-- single metadata for a file
CREATE TABLE IF NOT EXISTS tracks_metadata(
    id INTEGER PRIMARY KEY,
    -- the file this metadata is for; if the related file is dropped, drop this too
    file INTEGER NOT NULL UNIQUE REFERENCES tracks(id) ON DELETE CASCADE,
    -- can be null if not present or cannot be parsed
    title TEXT,
    -- can be null if not present or cannot be parsed
    genre TEXT,
    -- what will be shown for the author field, example "AuthorA feat. AuthorB" (but for linking use the artists / tracks_artists tables)
    author_display TEXT
);

-- the table for all artists
-- this is so that tracks with "ArtistA feat. ArtistB" can be searched for either
CREATE TABLE IF NOT EXISTS artists(
    id INTEGER PRIMARY KEY,
    -- artist is used as a identifier, if not present, it should not be added to the database
    artist TEXT NOT NULL UNIQUE,
    -- the date this artist was added to the database
    added_at DATE
);

-- relation table for a file's artist
-- entry will get deleted if the artist is dropped or the file is dropped
CREATE TABLE IF NOT EXISTS tracks_artists(
    id INTEGER PRIMARY KEY,
    file INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    artist INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE
);

-- the table for all albums
-- Note: not unique, there *could* be ones with the same name
CREATE TABLE IF NOT EXISTS albums(
    id INTEGER PRIMARY KEY,
    -- the title of the album, if it failed to parse, no album should be added
    title TEXT NOT NULL,
    -- what will be shown for the author field, example "AuthorA feat. AuthorB" (but for linking use the artists / albums_artist tables)
    author_display TEXT
);

-- relation table for a album's artist
-- this is so that albums with "ArtistA feat. ArtistB" can be searched for either
-- entry will get deleted if either the artist or the album get dropped
CREATE TABLE IF NOT EXISTS albums_artist(
    id INTEGER PRIMARY KEY,
    album INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    artist INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE
);

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
