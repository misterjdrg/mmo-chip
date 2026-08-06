CREATE TABLE files (
    id TEXT,
    name TEXT,
    mime TEXT,
    bytes BLOB,
    created_at TEXT,

    PRIMARY KEY(id)
);
CREATE TABLE dies (
    id TEXT,
    name TEXT,
    original_file_id TEXT,
    width INTEGER,
    height INTEGER,
    tile_size INTEGER,
    max_zoom_level INTEGER,
    zoom_levels TEXT,

    annotation_version INTEGER,
    annotation_revision INTEGER,
    ml_config TEXT,

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

CREATE TABLE die_params (
    id TEXT,
    die_id TEXT,
    kind TEXT,
    content TEXT,

    PRIMARY KEY(id, die_id, kind)
);
