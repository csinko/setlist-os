-- albums = primary user-visible entity
CREATE TABLE albums (
    id           UUID         PRIMARY KEY,
    title        TEXT,
    artist       TEXT,
    year         INT,
    kind         TEXT NOT NULL DEFAULT 'unknown',   -- studio | concert | hybrid | unknown
    source       JSONB        NOT NULL,             -- {type:"upload", …} / {type:"archive", …}
    imported_at  TIMESTAMPTZ  DEFAULT now()
);

-- logical tracks
CREATE TABLE tracks (
    id          UUID PRIMARY KEY,
    album_id    UUID REFERENCES albums(id) ON DELETE CASCADE,
    disc        INT DEFAULT 1,
    "index"     INT,
    title       TEXT,
    duration_sec INT,
    UNIQUE(album_id, disc, "index")
);

-- physical files
CREATE TABLE files (
    id          UUID PRIMARY KEY,
    track_id    UUID REFERENCES tracks(id) ON DELETE CASCADE,
    path        TEXT UNIQUE NOT NULL,
    codec       TEXT,
    status      TEXT NOT NULL DEFAULT 'NEW',     -- NEW→FP_DONE→TAG_DONE→READY
    fp_done_at  TIMESTAMPTZ,
    tagged_at   TIMESTAMPTZ,
    inserted_at TIMESTAMPTZ DEFAULT now()
);

-- durable job table
CREATE TABLE jobs (
    id            BIGSERIAL PRIMARY KEY,
    stage         TEXT    NOT NULL,         -- matches shared::Stage
    payload       JSONB   NOT NULL,
    status        TEXT    NOT NULL DEFAULT 'queued',  -- queued | running | done | error
    next_attempt  TIMESTAMPTZ DEFAULT now(),
    retry_count   INT     NOT NULL DEFAULT 0,
    last_error    TEXT
);

CREATE INDEX jobs_next_attempt ON jobs(next_attempt) WHERE status = 'queued';

