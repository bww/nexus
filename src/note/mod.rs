use std::fmt::{self, Display};

use rusqlite::{Connection, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json;

use crate::cli;
use crate::error;

pub mod cmd;

pub fn format_id(id: i32) -> String {
  format!("#{}", id)
}

#[derive(Debug, Serialize)]
pub struct NoteSummary<'a> {
  pub id: i32,
  pub creator_id: &'a String,
  pub commit_sha: &'a Option<String>,
  pub summary: &'a String,
  pub detail: &'a Option<String>,
  pub created_at: &'a DateTime<Utc>,
  pub updated_at: &'a DateTime<Utc>,
}

impl<'a> NoteSummary<'a> {
  pub fn formatted<'b>(&'b self, format: &'b cli::Format) -> cli::Formatted<'b, NoteSummary<'b>> {
    cli::Formatted { value: self, format }
  }

  pub fn fmt_text(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}. {}", format_id(self.id), self.summary)
  }

  pub fn fmt_json(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match serde_json::to_string(self) {
      Ok(json) => write!(f, "{}", json),
      Err(_)   => Err(fmt::Error),
    }
  }
}

impl Display for cli::Formatted<'_, NoteSummary<'_>> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.format {
      cli::Format::Text => self.value.fmt_text(f),
      cli::Format::JSON => self.value.fmt_json(f),
    }
  }
}

#[derive(Debug, Serialize)]
pub struct Note {
  pub id: i32,
  pub creator_id: String,
  pub commit_sha: Option<String>,
  pub summary: String,
  pub detail: Option<String>,
  pub data: Option<Vec<u8>>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl Note {
  pub fn init_db(conn: &rusqlite::Connection) -> Result<(), error::Error> {
    conn.execute("
      CREATE TABLE IF NOT EXISTS note (
        id         INTEGER PRIMARY KEY,
        creator_id TEXT NOT NULL,
        commit_sha TEXT,
        summary    TEXT NOT NULL,
        detail     TEXT,
        data       BLOB,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
      )",
      (),
    )?;
    Ok(())
  }

  pub fn summary(&self) -> NoteSummary<'_> {
    NoteSummary{
      id: self.id,
      creator_id: &self.creator_id,
      commit_sha: &self.commit_sha,
      summary: &self.summary,
      detail: &self.detail,
      created_at: &self.created_at,
      updated_at: &self.updated_at,
    }
  }

  pub fn formatted<'a>(&'a self, format: &'a cli::Format) -> cli::Formatted<'a, Note> {
    cli::Formatted { value: self, format }
  }

  pub fn fmt_text(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}. {}", format_id(self.id), self.summary)
  }

  pub fn fmt_json(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match serde_json::to_string(self) {
      Ok(json) => write!(f, "{}", json),
      Err(_)   => Err(fmt::Error),
    }
  }
}

impl Display for cli::Formatted<'_, Note> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.format {
      cli::Format::Text => self.value.fmt_text(f),
      cli::Format::JSON => self.value.fmt_json(f),
    }
  }
}

fn from_row(_conn: &Connection) -> impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<Note> + '_ {
  |row| {
    Ok(Note {
      id: row.get(0)?,
      creator_id: row.get(1)?,
      commit_sha: row.get(2)?,
      summary: row.get(3)?,
      detail: row.get(4)?,
      data: row.get(5)?,
      created_at: row.get(6)?,
      updated_at: row.get(7)?,
    })
  }
}

fn fetch(conn: &Connection, id: i32) -> Result<Note, error::Error> {
  let mut stmt = conn.prepare("
    SELECT id, creator_id, commit_sha, summary, detail, data, created_at, updated_at
    FROM note
    WHERE id = ?1")?;

  Ok(stmt.query_one(rusqlite::params![id], from_row(&conn))?)
}
