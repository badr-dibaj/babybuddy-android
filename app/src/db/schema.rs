/// SQL DDL statements for BabyBuddy database schema.
/// All tables use TEXT primary keys (UUID v4) for portability.

pub const CREATE_BABIES: &str = "
CREATE TABLE IF NOT EXISTS babies (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    birth_date  TEXT NOT NULL,   -- ISO-8601 date YYYY-MM-DD
    photo_path  TEXT,            -- optional local file path
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub const CREATE_FEEDINGS: &str = "
CREATE TABLE IF NOT EXISTS feedings (
    id          TEXT PRIMARY KEY NOT NULL,
    baby_id     TEXT NOT NULL REFERENCES babies(id) ON DELETE CASCADE,
    started_at  TEXT NOT NULL,   -- ISO-8601 datetime
    ended_at    TEXT,            -- nullable for ongoing
    method      TEXT NOT NULL,   -- 'left breast' | 'right breast' | 'both breasts' | 'bottle' | 'solid food' | 'fortified breast milk'
    type_       TEXT NOT NULL,   -- 'breast milk' | 'formula' | 'fortified breast milk' | 'solid food'
    amount      REAL,            -- ml or g depending on type
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub const CREATE_DIAPERS: &str = "
CREATE TABLE IF NOT EXISTS diapers (
    id          TEXT PRIMARY KEY NOT NULL,
    baby_id     TEXT NOT NULL REFERENCES babies(id) ON DELETE CASCADE,
    time        TEXT NOT NULL,   -- ISO-8601 datetime
    wet         INTEGER NOT NULL DEFAULT 0,   -- boolean 0/1
    solid       INTEGER NOT NULL DEFAULT 0,   -- boolean 0/1
    color       TEXT,            -- 'black' | 'brown' | 'green' | 'yellow' | 'orange' | 'red' | 'white'
    amount      TEXT,            -- 'small' | 'medium' | 'large'
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub const CREATE_WEIGHT: &str = "
CREATE TABLE IF NOT EXISTS weight (
    id          TEXT PRIMARY KEY NOT NULL,
    baby_id     TEXT NOT NULL REFERENCES babies(id) ON DELETE CASCADE,
    date_       TEXT NOT NULL,   -- ISO-8601 date
    weight      REAL NOT NULL,   -- kg
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub const CREATE_HEIGHT: &str = "
CREATE TABLE IF NOT EXISTS height (
    id          TEXT PRIMARY KEY NOT NULL,
    baby_id     TEXT NOT NULL REFERENCES babies(id) ON DELETE CASCADE,
    date_       TEXT NOT NULL,   -- ISO-8601 date
    height      REAL NOT NULL,   -- cm
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub const CREATE_HEAD_CIRCUMFERENCE: &str = "
CREATE TABLE IF NOT EXISTS head_circumference (
    id              TEXT PRIMARY KEY NOT NULL,
    baby_id         TEXT NOT NULL REFERENCES babies(id) ON DELETE CASCADE,
    date_           TEXT NOT NULL,   -- ISO-8601 date
    head_circumference REAL NOT NULL, -- cm
    notes           TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub const CREATE_TEETH: &str = "
CREATE TABLE IF NOT EXISTS teeth (
    id          TEXT PRIMARY KEY NOT NULL,
    baby_id     TEXT NOT NULL REFERENCES babies(id) ON DELETE CASCADE,
    date_       TEXT NOT NULL,   -- ISO-8601 date
    tooth_id    INTEGER NOT NULL, -- 0-19 (primary teeth numbering)
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(baby_id, tooth_id)
);";

pub const CREATE_MEDICATIONS: &str = "
CREATE TABLE IF NOT EXISTS medications (
    id          TEXT PRIMARY KEY NOT NULL,
    baby_id     TEXT NOT NULL REFERENCES babies(id) ON DELETE CASCADE,
    time        TEXT NOT NULL,   -- ISO-8601 datetime
    name        TEXT NOT NULL,   -- drug name e.g. 'Paracetamol'
    dosage      REAL,            -- numeric amount
    dosage_unit TEXT,            -- 'mg' | 'ml' | 'mcg' | 'mg/kg' | 'IU' | 'other'
    notes       TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub const CREATE_TIMERS: &str = "
CREATE TABLE IF NOT EXISTS timers (
    id          TEXT PRIMARY KEY NOT NULL,
    baby_id     TEXT NOT NULL REFERENCES babies(id) ON DELETE CASCADE,
    name        TEXT NOT NULL DEFAULT 'Timer',
    start       TEXT NOT NULL,   -- ISO-8601 datetime
    end_        TEXT,            -- nullable while running
    active      INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);";

/// All schema creation statements in dependency order.
pub const ALL_SCHEMAS: &[&str] = &[
    CREATE_BABIES,
    CREATE_FEEDINGS,
    CREATE_DIAPERS,
    CREATE_WEIGHT,
    CREATE_HEIGHT,
    CREATE_HEAD_CIRCUMFERENCE,
    CREATE_TEETH,
    CREATE_MEDICATIONS,
    CREATE_TIMERS,
];
