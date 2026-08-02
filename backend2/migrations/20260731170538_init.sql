CREATE TABLE files (
    id TEXT,
    name TEXT,
    mime TEXT,
    content BLOB,
    created_at TEXT,

    PRIMARY KEY(id)
);
CREATE TABLE dies (
    id TEXT,
    name TEXT,
    original_file_id TEXT,
    width INTEGER,
    height INTEGER,
    max_zoom_level INTEGER,
    zoom_levels INTEGER,

    created_at TEXT,
    updated_at TEXT,

    PRIMARY KEY(id)
);

CREATE TABLE tiles (
    die_id TEXT,
    z INTEGER,
    x INTEGER,
    y INTEGER,
    file_id TEXT,

    created_at TEXT,

    PRIMARY KEY(die_id, z, x, y)
);

CREATE TABLE jobs (
    id TEXT,
    kind TEXT,
    status TEXT,

    created_at TEXT,
    updated_at TEXT,
    started_at TEXT,
    finished_at TEXT,

    PRIMARY KEY(id)
);
