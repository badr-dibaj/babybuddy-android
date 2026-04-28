pub mod models;
pub mod schema;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use models::*;
use rusqlite::{params, Connection, Result as SqlResult};
use schema::ALL_SCHEMAS;
use uuid::Uuid;

// ─── Database ────────────────────────────────────────────────────────────────

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) the SQLite database at the given path.
    pub fn open(path: &str) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        // Enable foreign key enforcement and WAL mode for performance.
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Create all tables if they don't exist yet.
    fn init_schema(&self) -> SqlResult<()> {
        for stmt in ALL_SCHEMAS {
            self.conn.execute_batch(stmt)?;
        }
        Ok(())
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn new_id() -> String {
        Uuid::new_v4().to_string()
    }

    fn parse_dt(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                // fallback: treat as naive UTC
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                    .map(|ndt| Utc.from_utc_datetime(&ndt))
            })
            .unwrap_or_else(|_| Utc::now())
    }

    fn parse_date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap_or_else(|_| Utc::now().date_naive())
    }

    // ── Babies ────────────────────────────────────────────────────────────────

    pub fn get_babies(&self) -> SqlResult<Vec<Baby>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, birth_date, photo_path, created_at, updated_at
             FROM babies ORDER BY birth_date DESC",
        )?;
        let babies = stmt
            .query_map([], |row| {
                Ok(Baby {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    birth_date: {
                        let s: String = row.get(2)?;
                        Ok::<_, rusqlite::Error>(Self::parse_date(&s))
                    }
                    .unwrap(),
                    photo_path: row.get(3)?,
                    created_at: {
                        let s: String = row.get(4)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                    updated_at: {
                        let s: String = row.get(5)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(babies)
    }

    pub fn create_baby(&self, name: &str, birth_date: NaiveDate, photo_path: Option<&str>) -> SqlResult<Baby> {
        let id = Self::new_id();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO babies (id, name, birth_date, photo_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![
                id,
                name,
                birth_date.format("%Y-%m-%d").to_string(),
                photo_path,
                now.to_rfc3339(),
            ],
        )?;
        Ok(Baby {
            id,
            name: name.to_owned(),
            birth_date,
            photo_path: photo_path.map(str::to_owned),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_baby(&self, id: &str, name: &str, birth_date: NaiveDate) -> SqlResult<()> {
        let now = Utc::now();
        self.conn.execute(
            "UPDATE babies SET name=?1, birth_date=?2, updated_at=?3 WHERE id=?4",
            params![name, birth_date.format("%Y-%m-%d").to_string(), now.to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn delete_baby(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM babies WHERE id=?1", params![id])?;
        Ok(())
    }

    // ── Feedings ──────────────────────────────────────────────────────────────

    pub fn get_feedings(&self, baby_id: &str, limit: usize) -> SqlResult<Vec<Feeding>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, baby_id, started_at, ended_at, method, type_, amount, notes, created_at
             FROM feedings WHERE baby_id=?1 ORDER BY started_at DESC LIMIT ?2",
        )?;
        let feedings = stmt
            .query_map(params![baby_id, limit as i64], |row| {
                Ok(Feeding {
                    id: row.get(0)?,
                    baby_id: row.get(1)?,
                    started_at: {
                        let s: String = row.get(2)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                    ended_at: {
                        let s: Option<String> = row.get(3)?;
                        Ok::<_, rusqlite::Error>(s.map(|v| Self::parse_dt(&v)))
                    }
                    .unwrap(),
                    method: {
                        let s: String = row.get(4)?;
                        Ok::<_, rusqlite::Error>(FeedingMethod::from_str(&s))
                    }
                    .unwrap(),
                    type_: row.get(5)?,
                    amount: row.get(6)?,
                    notes: row.get(7)?,
                    created_at: {
                        let s: String = row.get(8)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(feedings)
    }

    pub fn create_feeding(
        &self,
        baby_id: &str,
        started_at: DateTime<Utc>,
        ended_at: Option<DateTime<Utc>>,
        method: &FeedingMethod,
        type_: &str,
        amount: Option<f64>,
        notes: Option<&str>,
    ) -> SqlResult<Feeding> {
        let id = Self::new_id();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO feedings (id, baby_id, started_at, ended_at, method, type_, amount, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                baby_id,
                started_at.to_rfc3339(),
                ended_at.map(|e| e.to_rfc3339()),
                method.as_str(),
                type_,
                amount,
                notes,
                now.to_rfc3339(),
            ],
        )?;
        Ok(Feeding {
            id,
            baby_id: baby_id.to_owned(),
            started_at,
            ended_at,
            method: method.clone(),
            type_: type_.to_owned(),
            amount,
            notes: notes.map(str::to_owned),
            created_at: now,
        })
    }

    pub fn delete_feeding(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM feedings WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn count_feedings_today(&self, baby_id: &str) -> SqlResult<usize> {
        let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM feedings WHERE baby_id=?1 AND date(started_at)=?2",
            params![baby_id, today],
            |r| r.get(0),
        )?;
        Ok(count as usize)
    }

    // ── Diapers ───────────────────────────────────────────────────────────────

    pub fn get_diapers(&self, baby_id: &str, limit: usize) -> SqlResult<Vec<Diaper>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, baby_id, time, wet, solid, color, amount, notes, created_at
             FROM diapers WHERE baby_id=?1 ORDER BY time DESC LIMIT ?2",
        )?;
        let diapers = stmt
            .query_map(params![baby_id, limit as i64], |row| {
                Ok(Diaper {
                    id: row.get(0)?,
                    baby_id: row.get(1)?,
                    time: {
                        let s: String = row.get(2)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                    wet: {
                        let v: i32 = row.get(3)?;
                        Ok::<_, rusqlite::Error>(v != 0)
                    }
                    .unwrap(),
                    solid: {
                        let v: i32 = row.get(4)?;
                        Ok::<_, rusqlite::Error>(v != 0)
                    }
                    .unwrap(),
                    color: row.get(5)?,
                    amount: row.get(6)?,
                    notes: row.get(7)?,
                    created_at: {
                        let s: String = row.get(8)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(diapers)
    }

    pub fn create_diaper(
        &self,
        baby_id: &str,
        time: DateTime<Utc>,
        wet: bool,
        solid: bool,
        color: Option<&str>,
        amount: Option<&str>,
        notes: Option<&str>,
    ) -> SqlResult<Diaper> {
        let id = Self::new_id();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO diapers (id, baby_id, time, wet, solid, color, amount, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                baby_id,
                time.to_rfc3339(),
                wet as i32,
                solid as i32,
                color,
                amount,
                notes,
                now.to_rfc3339(),
            ],
        )?;
        Ok(Diaper {
            id,
            baby_id: baby_id.to_owned(),
            time,
            wet,
            solid,
            color: color.map(str::to_owned),
            amount: amount.map(str::to_owned),
            notes: notes.map(str::to_owned),
            created_at: now,
        })
    }

    pub fn delete_diaper(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM diapers WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn count_diapers_today(&self, baby_id: &str) -> SqlResult<usize> {
        let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM diapers WHERE baby_id=?1 AND date(time)=?2",
            params![baby_id, today],
            |r| r.get(0),
        )?;
        Ok(count as usize)
    }

    // ── Weight ────────────────────────────────────────────────────────────────

    pub fn get_weights(&self, baby_id: &str) -> SqlResult<Vec<Weight>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, baby_id, date_, weight, notes, created_at
             FROM weight WHERE baby_id=?1 ORDER BY date_ ASC",
        )?;
        let rows = stmt
            .query_map(params![baby_id], |row| {
                Ok(Weight {
                    id: row.get(0)?,
                    baby_id: row.get(1)?,
                    date_: {
                        let s: String = row.get(2)?;
                        Ok::<_, rusqlite::Error>(Self::parse_date(&s))
                    }
                    .unwrap(),
                    weight: row.get(3)?,
                    notes: row.get(4)?,
                    created_at: {
                        let s: String = row.get(5)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn create_weight(&self, baby_id: &str, date: NaiveDate, weight: f64, notes: Option<&str>) -> SqlResult<Weight> {
        let id = Self::new_id();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO weight (id, baby_id, date_, weight, notes, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![id, baby_id, date.format("%Y-%m-%d").to_string(), weight, notes, now.to_rfc3339()],
        )?;
        Ok(Weight { id, baby_id: baby_id.to_owned(), date_: date, weight, notes: notes.map(str::to_owned), created_at: now })
    }

    pub fn delete_weight(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM weight WHERE id=?1", params![id])?;
        Ok(())
    }

    // ── Height ────────────────────────────────────────────────────────────────

    pub fn get_heights(&self, baby_id: &str) -> SqlResult<Vec<Height>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, baby_id, date_, height, notes, created_at
             FROM height WHERE baby_id=?1 ORDER BY date_ ASC",
        )?;
        let rows = stmt
            .query_map(params![baby_id], |row| {
                Ok(Height {
                    id: row.get(0)?,
                    baby_id: row.get(1)?,
                    date_: {
                        let s: String = row.get(2)?;
                        Ok::<_, rusqlite::Error>(Self::parse_date(&s))
                    }
                    .unwrap(),
                    height: row.get(3)?,
                    notes: row.get(4)?,
                    created_at: {
                        let s: String = row.get(5)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn create_height(&self, baby_id: &str, date: NaiveDate, height: f64, notes: Option<&str>) -> SqlResult<Height> {
        let id = Self::new_id();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO height (id, baby_id, date_, height, notes, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![id, baby_id, date.format("%Y-%m-%d").to_string(), height, notes, now.to_rfc3339()],
        )?;
        Ok(Height { id, baby_id: baby_id.to_owned(), date_: date, height, notes: notes.map(str::to_owned), created_at: now })
    }

    pub fn delete_height(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM height WHERE id=?1", params![id])?;
        Ok(())
    }

    // ── Head Circumference ────────────────────────────────────────────────────

    pub fn get_head_circumferences(&self, baby_id: &str) -> SqlResult<Vec<HeadCircumference>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, baby_id, date_, head_circumference, notes, created_at
             FROM head_circumference WHERE baby_id=?1 ORDER BY date_ ASC",
        )?;
        let rows = stmt
            .query_map(params![baby_id], |row| {
                Ok(HeadCircumference {
                    id: row.get(0)?,
                    baby_id: row.get(1)?,
                    date_: {
                        let s: String = row.get(2)?;
                        Ok::<_, rusqlite::Error>(Self::parse_date(&s))
                    }
                    .unwrap(),
                    head_circumference: row.get(3)?,
                    notes: row.get(4)?,
                    created_at: {
                        let s: String = row.get(5)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn create_head_circumference(
        &self, baby_id: &str, date: NaiveDate, hc: f64, notes: Option<&str>,
    ) -> SqlResult<HeadCircumference> {
        let id = Self::new_id();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO head_circumference (id, baby_id, date_, head_circumference, notes, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![id, baby_id, date.format("%Y-%m-%d").to_string(), hc, notes, now.to_rfc3339()],
        )?;
        Ok(HeadCircumference { id, baby_id: baby_id.to_owned(), date_: date, head_circumference: hc, notes: notes.map(str::to_owned), created_at: now })
    }

    pub fn delete_head_circumference(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM head_circumference WHERE id=?1", params![id])?;
        Ok(())
    }

    // ── Teeth ─────────────────────────────────────────────────────────────────

    pub fn get_teeth(&self, baby_id: &str) -> SqlResult<Vec<Tooth>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, baby_id, date_, tooth_id, notes, created_at
             FROM teeth WHERE baby_id=?1 ORDER BY date_ ASC",
        )?;
        let rows = stmt
            .query_map(params![baby_id], |row| {
                Ok(Tooth {
                    id: row.get(0)?,
                    baby_id: row.get(1)?,
                    date_: {
                        let s: String = row.get(2)?;
                        Ok::<_, rusqlite::Error>(Self::parse_date(&s))
                    }
                    .unwrap(),
                    tooth_id: row.get(3)?,
                    notes: row.get(4)?,
                    created_at: {
                        let s: String = row.get(5)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn create_tooth(&self, baby_id: &str, date: NaiveDate, tooth_id: i32, notes: Option<&str>) -> SqlResult<Tooth> {
        let id = Self::new_id();
        let now = Utc::now();
        self.conn.execute(
            "INSERT OR IGNORE INTO teeth (id, baby_id, date_, tooth_id, notes, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![id, baby_id, date.format("%Y-%m-%d").to_string(), tooth_id, notes, now.to_rfc3339()],
        )?;
        Ok(Tooth { id, baby_id: baby_id.to_owned(), date_: date, tooth_id, notes: notes.map(str::to_owned), created_at: now })
    }

    pub fn delete_tooth(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM teeth WHERE id=?1", params![id])?;
        Ok(())
    }

    // ── Medications ───────────────────────────────────────────────────────────

    pub fn get_medications(&self, baby_id: &str, limit: usize) -> SqlResult<Vec<Medication>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, baby_id, time, name, dosage, dosage_unit, notes, created_at
             FROM medications WHERE baby_id=?1 ORDER BY time DESC LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![baby_id, limit as i64], |row| {
                Ok(Medication {
                    id: row.get(0)?,
                    baby_id: row.get(1)?,
                    time: {
                        let s: String = row.get(2)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                    name: row.get(3)?,
                    dosage: row.get(4)?,
                    dosage_unit: row.get(5)?,
                    notes: row.get(6)?,
                    created_at: {
                        let s: String = row.get(7)?;
                        Ok::<_, rusqlite::Error>(Self::parse_dt(&s))
                    }
                    .unwrap(),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn create_medication(
        &self,
        baby_id: &str,
        time: DateTime<Utc>,
        name: &str,
        dosage: Option<f64>,
        dosage_unit: Option<&str>,
        notes: Option<&str>,
    ) -> SqlResult<Medication> {
        let id = Self::new_id();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO medications (id, baby_id, time, name, dosage, dosage_unit, notes, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![id, baby_id, time.to_rfc3339(), name, dosage, dosage_unit, notes, now.to_rfc3339()],
        )?;
        Ok(Medication {
            id,
            baby_id: baby_id.to_owned(),
            time,
            name: name.to_owned(),
            dosage,
            dosage_unit: dosage_unit.map(str::to_owned),
            notes: notes.map(str::to_owned),
            created_at: now,
        })
    }

    pub fn delete_medication(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM medications WHERE id=?1", params![id])?;
        Ok(())
    }

    // ── Dashboard ─────────────────────────────────────────────────────────────

    pub fn get_day_summary(&self, baby_id: &str, baby_name: &str) -> SqlResult<DaySummary> {
        let feedings_today = self.count_feedings_today(baby_id)?;
        let diapers_today = self.count_diapers_today(baby_id)?;

        let last_feeding: Option<DateTime<Utc>> = self
            .conn
            .query_row(
                "SELECT started_at FROM feedings WHERE baby_id=?1 ORDER BY started_at DESC LIMIT 1",
                params![baby_id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .map(|s| Self::parse_dt(&s));

        let last_diaper: Option<DateTime<Utc>> = self
            .conn
            .query_row(
                "SELECT time FROM diapers WHERE baby_id=?1 ORDER BY time DESC LIMIT 1",
                params![baby_id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .map(|s| Self::parse_dt(&s));

        let latest_weight: Option<f64> = self
            .conn
            .query_row(
                "SELECT weight FROM weight WHERE baby_id=?1 ORDER BY date_ DESC LIMIT 1",
                params![baby_id],
                |r| r.get(0),
            )
            .ok();

        let latest_height: Option<f64> = self
            .conn
            .query_row(
                "SELECT height FROM height WHERE baby_id=?1 ORDER BY date_ DESC LIMIT 1",
                params![baby_id],
                |r| r.get(0),
            )
            .ok();

        Ok(DaySummary {
            baby_name: baby_name.to_owned(),
            feedings_today,
            diapers_today,
            last_feeding,
            last_diaper,
            latest_weight,
            latest_height,
        })
    }
}
