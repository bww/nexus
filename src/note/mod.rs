use std::fmt::{self, Display};

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

  pub fn formatted<'a>(&'a self, format: &'a cli::Format) -> cli::Formatted<'a, Note> {
    cli::Formatted { value: self, format }
  }

  pub fn fmt_text(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} {}", format_id(self.id), self.summary)
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
