use std::error::Error;
use rusqlite::{Connection, Result as SqliteResult};
use chrono::{DateTime, Local};

use crate::models::bugreport::logcat::LogcatLine;

pub trait LogcatRepository {
    fn insert(&self, log: &LogcatLine) -> Result<i64, Box<dyn Error>>;
    fn insert_batch(&self, logs: &[LogcatLine]) -> Result<Vec<i64>, Box<dyn Error>>;
    fn update(&self, log: &LogcatLine) -> Result<(), Box<dyn Error>>;
    fn delete(&self, id: i64) -> Result<(), Box<dyn Error>>;
    fn find_all(&self) -> Result<Vec<LogcatLine>, Box<dyn Error>>;
    fn find_by_id(&self, id: i64) -> Result<Option<LogcatLine>, Box<dyn Error>>;
}

impl std::fmt::Debug for dyn LogcatRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LogcatRepository")
    }
}

#[derive(Debug)]
pub struct SqliteLogcatRepository {
    conn: Connection,
}

impl SqliteLogcatRepository {
    pub fn new_in_memory() -> SqliteResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS logcat_lines (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                user TEXT NOT NULL,
                pid INTEGER NOT NULL,
                tid INTEGER NOT NULL,
                level TEXT NOT NULL CHECK(length(level) = 1),
                tag TEXT NOT NULL,
                message TEXT NOT NULL
            )",
            [],
        )?;
        Ok(SqliteLogcatRepository { conn })
    }
}

impl LogcatRepository for SqliteLogcatRepository {
    fn insert(&self, log: &LogcatLine) -> Result<i64, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO logcat_lines (timestamp, user, pid, tid, level, tag, message)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )?;
        
        let timestamp_str = log.timestamp.to_rfc3339();
        let level_str = log.level.to_string();
        
        stmt.execute([
            timestamp_str,
            log.user.clone(),
            log.pid.to_string(),
            log.tid.to_string(),
            level_str,
            log.tag.clone(),
            log.message.clone(),
        ])?;
        
        Ok(self.conn.last_insert_rowid())
    }

    fn insert_batch(&self, logs: &[LogcatLine]) -> Result<Vec<i64>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO logcat_lines (timestamp, user, pid, tid, level, tag, message)https://xiaomi.f.mioffice.cn/wiki/SosTw6JHviW1omkcIlSkrrH045f
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )?;
        
        let mut ids = Vec::with_capacity(logs.len());
        for log in logs {
            let timestamp_str = log.timestamp.to_rfc3339();
            let level_str = log.level.to_string();
            stmt.execute([
                timestamp_str,
                log.user.clone(),
                log.pid.to_string(),
                log.tid.to_string(),
                level_str,
                log.tag.clone(),
                log.message.clone(),
            ])?;
            ids.push(self.conn.last_insert_rowid());
        }

        Ok(ids)
    }

    fn update(&self, log: &LogcatLine) -> Result<(), Box<dyn Error>> {
        let id = log.id.ok_or("Cannot update log without ID")?;
        let mut stmt = self.conn.prepare(
            "UPDATE logcat_lines SET timestamp = ?, user = ?, pid = ?, tid = ?, 
             level = ?, tag = ?, message = ? WHERE id = ?"
        )?;
        
        let timestamp_str = log.timestamp.to_rfc3339();
        let level_str = log.level.to_string();
        
        stmt.execute([
            timestamp_str,
            log.user.clone(),
            log.pid.to_string(),
            log.tid.to_string(),
            level_str,
            log.tag.clone(),
            log.message.clone(),
            id.to_string(),
        ])?;
        Ok(())
    }

    fn delete(&self, id: i64) -> Result<(), Box<dyn Error>> {
        let mut stmt = self.conn.prepare("DELETE FROM logcat_lines WHERE id = ?")?;
        stmt.execute([id])?;
        Ok(())
    }

    fn find_all(&self) -> Result<Vec<LogcatLine>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, user, pid, tid, level, tag, message 
             FROM logcat_lines"
        )?;
        
        let logs = stmt.query_map([], |row| {
            Ok(LogcatLine {
                id: Some(row.get(0)?),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .map(|dt| dt.with_timezone(&Local))
                    .unwrap(),
                user: row.get(2)?,
                pid: row.get(3)?,
                tid: row.get(4)?,
                level: row.get::<_, String>(5)?.chars().next().unwrap(),
                tag: row.get(6)?,
                message: row.get(7)?,
            })
        })?.collect::<SqliteResult<Vec<_>>>()?;
        
        Ok(logs)
    }

    fn find_by_id(&self, id: i64) -> Result<Option<LogcatLine>, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, user, pid, tid, level, tag, message 
             FROM logcat_lines WHERE id = ?"
        )?;
        
        let mut rows = stmt.query([id])?;
        Ok(rows.next()?.map(|row| {
            Ok::<LogcatLine, Box<dyn Error>>(LogcatLine {
                id: Some(row.get(0)?),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .map(|dt| dt.with_timezone(&Local))
                    .unwrap(),
                user: row.get(2)?,
                pid: row.get(3)?,
                tid: row.get(4)?,
                level: row.get::<_, String>(5)?.chars().next().unwrap(),
                tag: row.get(6)?,
                message: row.get(7)?,
            })
        }).transpose()?)
    }
}

pub struct MockLogcatRepository {
    logs: Vec<LogcatLine>,
}

impl MockLogcatRepository {
    pub fn new() -> Self {
        MockLogcatRepository { logs: Vec::new() }
    }
}

impl LogcatRepository for MockLogcatRepository {
    fn insert(&self, log: &LogcatLine) -> Result<i64, Box<dyn Error>> {
        // Simplified mock behavior
        Ok(1)
    }

    fn insert_batch(&self, logs: &[LogcatLine]) -> Result<Vec<i64>, Box<dyn Error>> {
        // Simplified mock behavior
        Ok(vec![1; logs.len()])
    }

    fn find_all(&self) -> Result<Vec<LogcatLine>, Box<dyn Error>> {
        Ok(self.logs.clone())
    }

    fn find_by_id(&self, id: i64) -> Result<Option<LogcatLine>, Box<dyn Error>> {
        Ok(self.logs.iter().find(|log| log.id == Some(id)).cloned())
    }

    fn update(&self, _log: &LogcatLine) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn delete(&self, _id: i64) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

