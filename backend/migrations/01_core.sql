-- 01_core.sql  ── *initial* schema (safe to drop / recreate in dev)

-------------------------------------------------------------------------------
-- ENUM TYPES ─────────────────────────────────────────────────────────────────
-------------------------------------------------------------------------------
CREATE TYPE file_status   AS ENUM ('NEW', 'FP_DONE', 'TAG_DONE', 'READY', 'ERROR');
CREATE TYPE album_kind    AS ENUM ('studio', 'concert', 'hybrid', 'unknown');

-------------------------------------------------------------------------------
-- ALBUMS (high-level user entity) ────────────────────────────────────────────
-------------------------------------------------------------------------------
CREATE TABLE albums (
    id          UUID        PRIMARY KEY,
    title       TEXT,
    artist      TEXT,
    year        INT,
    kind        album_kind  NOT NULL DEFAULT 'unknown',
    source      JSONB       NOT NULL,     -- {type:"upload"|remote|scan, …}
    imported_at TIMESTAMPTZ DEFAULT now()
);

-------------------------------------------------------------------------------
-- TRACKS (logical songs) ─────────────────────────────────────────────────────
-------------------------------------------------------------------------------
CREATE TABLE tracks (
    id           UUID PRIMARY KEY,
    album_id     UUID REFERENCES albums(id) ON DELETE CASCADE,
    disc         INT  DEFAULT 1,
    "index"      INT,                -- 1-based within disc
    title        TEXT,
    duration_sec INT,               -- filled by Fingerprint
    UNIQUE(album_id, disc, "index")
);

-------------------------------------------------------------------------------
-- FILES (physical media objects) ─────────────────────────────────────────────
-------------------------------------------------------------------------------
CREATE TABLE files (
    id          UUID PRIMARY KEY,
    track_id    UUID REFERENCES tracks(id) ON DELETE CASCADE,
    path        TEXT UNIQUE NOT NULL,
    codec       TEXT        NOT NULL,              -- e.g. flac / mp3 / opus
    status      file_status NOT NULL DEFAULT 'NEW',
    fp_done_at  TIMESTAMPTZ,
    tagged_at   TIMESTAMPTZ,
    inserted_at TIMESTAMPTZ DEFAULT now()
);

-------------------------------------------------------------------------------
-- MATCHES (fingerprint → MusicBrainz) ────────────────────────────────────────
-------------------------------------------------------------------------------
-- Per-file/recording (Match-Track)
CREATE TABLE matches_track (
    file_id       UUID REFERENCES files(id)   ON DELETE CASCADE,
    mb_recording  UUID,             -- MusicBrainz Recording MBID
    score         REAL,             -- 0.0-1.0 or AcoustID score
    raw_json      JSONB NOT NULL,   -- provider response
    chosen        BOOL  DEFAULT FALSE,
    PRIMARY KEY (file_id, mb_recording)
);

-- Optional per-album consensus (Match-Album)
CREATE TABLE matches_album (
    album_id    UUID REFERENCES albums(id)  ON DELETE CASCADE,
    mb_release  UUID,            -- MusicBrainz Release MBID
    confidence  REAL,
    raw_json    JSONB NOT NULL,
    PRIMARY KEY (album_id, mb_release)
);

-------------------------------------------------------------------------------
-- DURABLE JOB QUEUE  ─────────────────────────────────────────────────────────
-------------------------------------------------------------------------------
CREATE TABLE jobs (
    id            BIGSERIAL PRIMARY KEY,
    stage         TEXT    NOT NULL,  -- must map to shared::Stage snake-case
    payload       JSONB   NOT NULL,  -- JobEnvelope
    status        TEXT    NOT NULL DEFAULT 'queued',  -- queued|running|done|error
    next_attempt  TIMESTAMPTZ DEFAULT now(),
    retry_count   INT     NOT NULL DEFAULT 0,
    last_error    TEXT
);
CREATE INDEX jobs_next_attempt        ON jobs(next_attempt)
  WHERE status = 'queued';
CREATE INDEX jobs_stage_queued_status ON jobs(stage)
  WHERE status = 'queued';

