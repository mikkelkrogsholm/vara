use crate::*;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

/// Transactional local state. The host serializes access; it is not an OS paste transaction.
pub struct Store {
    db: Connection,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let db = Connection::open(path).map_err(Error::Storage)?;
        let store = Self { db };
        store.initialize()?;
        Ok(store)
    }
    fn initialize(&self) -> Result<()> {
        self.db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA secure_delete=ON; PRAGMA foreign_keys=ON;
            CREATE TABLE IF NOT EXISTS preferences (id INTEGER PRIMARY KEY CHECK(id=1), value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS results (id TEXT PRIMARY KEY,generation TEXT NOT NULL UNIQUE,created_at INTEGER NOT NULL,origin_app TEXT NOT NULL,value TEXT NOT NULL);
            CREATE INDEX IF NOT EXISTS results_date ON results(created_at DESC);
            CREATE TABLE IF NOT EXISTS recovery (id TEXT PRIMARY KEY,value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS staging (generation TEXT PRIMARY KEY,value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS cancelled (generation TEXT PRIMARY KEY);").map_err(Error::Storage)?;
        // A pending result has not been dispatched; an unknown OS handoff must never be replayed.
        for mut result in self.history(None, None)? {
            if result.delivery == Delivery::Pending {
                result.delivery = Delivery::ManualReady;
                self.update_delivery(&result.id, result.delivery)?;
            }
        }
        Ok(())
    }
    pub fn settings(&self) -> Result<Settings> {
        let value: Option<String> = self
            .db
            .query_row("SELECT value FROM preferences WHERE id=1", [], |r| r.get(0))
            .optional()
            .map_err(Error::Storage)?;
        let mut settings: Settings = value
            .map(|v| serde_json::from_str(&v).map_err(Error::from))
            .unwrap_or_else(|| Ok(Settings::default()))?;
        // New built-ins become available without replacing a user's selected engine.
        for engine in default_engines() {
            if !settings.engines.iter().any(|old| old.id == engine.id) {
                settings.engines.push(engine);
            }
        }
        Ok(settings)
    }
    pub fn save_settings(&self, s: &Settings) -> Result<()> {
        s.validate()?;
        let json = serde_json::to_string(s)?;
        self.db.execute("INSERT INTO preferences(id,value) VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET value=excluded.value",[json]).map_err(Error::Storage)?;
        Ok(())
    }
    pub fn cancel_generation(&mut self, generation: &str) -> Result<()> {
        let tx = self.db.transaction().map_err(Error::Storage)?;
        tx.execute(
            "INSERT OR IGNORE INTO cancelled(generation) VALUES(?1)",
            [generation],
        )
        .map_err(Error::Storage)?;
        tx.execute("DELETE FROM staging WHERE generation=?1", [generation])
            .map_err(Error::Storage)?;
        tx.commit().map_err(Error::Storage)?;
        Ok(())
    }
    fn is_cancelled(&self, generation: &str) -> Result<bool> {
        self.db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM cancelled WHERE generation=?1)",
                [generation],
                |r| r.get(0),
            )
            .map_err(Error::Storage)
    }
    pub fn stage(&self, s: &StagedText) -> Result<()> {
        if self.is_cancelled(&s.generation)? {
            return Err(Error::Cancelled);
        }
        self.db.execute("INSERT INTO staging(generation,value) VALUES(?1,?2) ON CONFLICT(generation) DO UPDATE SET value=excluded.value",params![s.generation,serde_json::to_string(s)?]).map_err(Error::Storage)?;
        Ok(())
    }
    pub fn staged(&self) -> Result<Vec<StagedText>> {
        self.read_values("SELECT value FROM staging", [])
    }
    pub fn remove_staged(&self, generation: &str) -> Result<()> {
        self.db
            .execute("DELETE FROM staging WHERE generation=?1", [generation])
            .map_err(Error::Storage)?;
        Ok(())
    }
    /// action.save_result: one generation commits at most once, including both text versions.
    pub fn commit(&mut self, result: &Dictation) -> Result<Dictation> {
        if self.is_cancelled(&result.generation)? {
            return Err(Error::Cancelled);
        }
        let tx = self.db.transaction().map_err(Error::Storage)?;
        let existing: Option<String> = tx
            .query_row(
                "SELECT value FROM results WHERE generation=?1",
                [&result.generation],
                |r| r.get(0),
            )
            .optional()
            .map_err(Error::Storage)?;
        if let Some(v) = existing {
            return Ok(serde_json::from_str(&v)?);
        }
        tx.execute(
            "INSERT INTO results(id,generation,created_at,origin_app,value) VALUES(?1,?2,?3,?4,?5)",
            params![
                result.id,
                result.generation,
                result.created_at,
                result.origin.app_id,
                serde_json::to_string(result)?
            ],
        )
        .map_err(Error::Storage)?;
        tx.execute(
            "DELETE FROM staging WHERE generation=?1",
            [&result.generation],
        )
        .map_err(Error::Storage)?;
        tx.commit().map_err(Error::Storage)?;
        Ok(result.clone())
    }
    pub fn history(&self, app: Option<&str>, query: Option<&str>) -> Result<Vec<Dictation>> {
        let all: Vec<Dictation> = self.read_values(
            "SELECT value FROM results ORDER BY created_at DESC, rowid DESC",
            [],
        )?;
        let query = query.map(str::to_lowercase);
        Ok(all
            .into_iter()
            .filter(|r| {
                app.is_none_or(|a| r.origin.app_id == a)
                    && query.as_ref().is_none_or(|q| {
                        r.original.to_lowercase().contains(q)
                            || r.processed
                                .as_ref()
                                .is_some_and(|p| p.to_lowercase().contains(q))
                    })
            })
            .collect())
    }
    pub fn result(&self, id: &str) -> Result<Option<Dictation>> {
        let v: Option<String> = self
            .db
            .query_row("SELECT value FROM results WHERE id=?1", [id], |r| r.get(0))
            .optional()
            .map_err(Error::Storage)?;
        v.map(|v| serde_json::from_str(&v).map_err(Error::from))
            .transpose()
    }
    pub fn update_delivery(&self, id: &str, status: Delivery) -> Result<()> {
        if let Some(mut r) = self.result(id)? {
            r.delivery = status;
            self.db
                .execute(
                    "UPDATE results SET value=?2 WHERE id=?1",
                    params![id, serde_json::to_string(&r)?],
                )
                .map_err(Error::Storage)?;
        }
        Ok(())
    }
    pub fn delete(&mut self, id: &str) -> Result<()> {
        let tx = self.db.transaction().map_err(Error::Storage)?;
        tx.execute("DELETE FROM results WHERE id=?1", [id])
            .map_err(Error::Storage)?;
        tx.commit().map_err(Error::Storage)?;
        self.checkpoint()?;
        Ok(())
    }
    pub fn retain(&mut self, now: i64, days: Option<u32>) -> Result<usize> {
        let Some(days) = days else { return Ok(0) };
        if days == 0 {
            return Err(Error::Invalid("Vælg et positivt antal dage.".into()));
        }
        let count = self
            .db
            .execute(
                "DELETE FROM results WHERE created_at<=?1",
                [now.saturating_sub(i64::from(days) * 86400)],
            )
            .map_err(Error::Storage)?;
        if count > 0 {
            self.checkpoint()?;
        }
        Ok(count)
    }
    pub fn recoveries(&self) -> Result<Vec<Recovery>> {
        self.read_values("SELECT value FROM recovery", [])
    }
    pub fn recovery(&self, id: &str) -> Result<Option<Recovery>> {
        Ok(self.recoveries()?.into_iter().find(|r| r.id == id))
    }
    pub fn put_recovery(&self, r: &Recovery) -> Result<()> {
        self.db.execute("INSERT INTO recovery(id,value) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET value=excluded.value",params![r.id,serde_json::to_string(r)?]).map_err(Error::Storage)?;
        Ok(())
    }
    pub fn delete_recovery(&self, id: &str) -> Result<()> {
        self.db
            .execute("DELETE FROM recovery WHERE id=?1", [id])
            .map_err(Error::Storage)?;
        Ok(())
    }
    fn read_values<T: serde::de::DeserializeOwned, P: rusqlite::Params>(
        &self,
        sql: &str,
        p: P,
    ) -> Result<Vec<T>> {
        let mut stmt = self.db.prepare(sql).map_err(Error::Storage)?;
        let values = stmt
            .query_map(p, |r| r.get::<_, String>(0))
            .map_err(Error::Storage)?;
        values
            .map(|v| Ok(serde_json::from_str(&v.map_err(Error::Storage)?)?))
            .collect()
    }
    fn checkpoint(&self) -> Result<()> {
        self.db
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
            .map_err(Error::Storage)?;
        Ok(())
    }
}
