use std::fmt::{self, Display};
use std::str::FromStr;

use chrono::{DateTime, Utc};

use crate::error;

pub mod cmd;

#[derive(Debug)]
pub enum State {
  Available,
  InProgress,
  Done,
}

impl State {
  fn from_text(text: &[u8]) -> rusqlite::types::FromSqlResult<Self> {
    let text = match str::from_utf8(text) {
      Ok(text) => text,
      Err(err) => return Err(rusqlite::types::FromSqlError::Utf8Error(err)),
    };
    match  State::from_str(text) {
      Ok(state) => Ok(state),
      Err(_)    => return Err(rusqlite::types::FromSqlError::InvalidType),
    }
  }
}

impl FromStr for State {
  type Err = String;
  fn from_str(input: &str) -> Result<Self, Self::Err> {
    match input {
      "available"   => Ok(State::Available),
      "in_progress" => Ok(State::InProgress),
      "done"        => Ok(State::Done),
      _             => Err(format!("Invalid state: {}", input)),
    }
  }
}

impl Display for State {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Available  => write!(f, "available"),
      Self::InProgress => write!(f, "in_progress"),
      Self::Done       => write!(f, "done"),
    }
  }
}

impl rusqlite::types::FromSql for State {
  fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
    match value {
      rusqlite::types::ValueRef::Text(text) => Ok(State::from_text(text)?),
      _                                     => Err(rusqlite::types::FromSqlError::InvalidType),
    }
  }
}

impl rusqlite::types::ToSql for State {
  fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
    Ok(rusqlite::types::ToSqlOutput::Owned(
      rusqlite::types::Value::Text(self.to_string())
    ))
  }
}

#[derive(Debug)]
pub struct Ticket {
  pub id: i32,
  pub state: State,
  pub summary: String,
  pub detail: Option<String>,
  pub data: Option<Vec<u8>>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl Ticket {
  pub fn init_db(conn: &rusqlite::Connection) -> Result<(), error::Error> {
    conn.execute(
      "CREATE TABLE IF NOT EXISTS ticket (
        id         INTEGER PRIMARY KEY,
        state      TEXT NOT NULL,
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
}
