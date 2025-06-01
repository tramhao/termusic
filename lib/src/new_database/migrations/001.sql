--- SECTION: Config

-- table for extra information in a key-value style
-- examples ("key: val_format"): "db_created_at: DATE", "last_used_version: TERMUSIC_VERSION"
CREATE TABLE IF NOT EXISTS config(
    key TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
);

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
    -- id INTEGER PRIMARY KEY,
    -- the track this metadata is for; if the related track is dropped, drop this too
    track INTEGER PRIMARY KEY NOT NULL UNIQUE REFERENCES tracks(id) ON DELETE CASCADE,
    -- can be null if not present or cannot be parsed
    title TEXT,
    -- can be null if not present or cannot be parsed
    genre TEXT,
    -- what will be shown for the artist field, example "ArtistA feat. ArtistB" (but for linking use the artists / tracks_artists tables)
    artist_display TEXT
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

-- relation table for a tracks's artist
-- entry will get deleted if the artist is dropped or the track is dropped
CREATE TABLE IF NOT EXISTS tracks_artists(
    track INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    artist INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    PRIMARY KEY (track, artist)
);

-- the table for all albums
-- Note: not unique, there *could* be ones with the same name
-- Note: "artist_display" or "albums_artists" should be used to match exact album
CREATE TABLE IF NOT EXISTS albums(
    id INTEGER PRIMARY KEY,
    -- the title of the album, if it failed to parse, no album should be added
    title TEXT NOT NULL,
    -- what will be shown for the artist field, example "ArtistA feat. ArtistB" (but for linking use the artists / tracks_artists tables)
    artist_display TEXT
);

-- relation table for a album's artist
-- this is so that albums with "ArtistA feat. ArtistB" can be searched for either
-- entry will get deleted if either the artist or the album get dropped
CREATE TABLE IF NOT EXISTS albums_artists(
    album INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    artist INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    PRIMARY KEY (album, artist)
);
