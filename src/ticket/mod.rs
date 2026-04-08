use std::fmt::{self, Display};
use std::str::FromStr;
use std::borrow::Borrow;

use rusqlite::{Connection, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json;

use crate::cli;
use crate::error;

pub mod cmd;

#[derive(Debug, Clone, Serialize)]
pub enum State {
  #[serde(rename = "available")]
  Available,
  #[serde(rename = "in_progress")]
  InProgress,
  #[serde(rename = "done")]
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

pub fn format_id(id: i32) -> String {
  format!("#{}", id)
}

pub fn format_ids<T: Borrow<i32>>(ids: &[T]) -> String {
  let mut buf = String::new();
  for id in ids {
    if buf.len() > 0 {
      buf.push_str(", ");
    }
    buf.push_str(&format_id(*id.borrow()));
  }
  buf
}

#[derive(Debug, Serialize)]
pub struct TicketSummary<'a> {
  pub id: i32,
  pub state: &'a State,
  pub roles: &'a Option<Vec<String>>,
  pub summary: &'a String,
  pub owner_id: &'a Option<String>,
  pub created_at: &'a DateTime<Utc>,
  pub updated_at: &'a DateTime<Utc>,
}

impl<'a> TicketSummary<'a> {
  pub fn formatted<'b>(&'b self, format: &'b cli::Format) -> cli::Formatted<'b, TicketSummary<'b>> {
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

impl Display for cli::Formatted<'_, TicketSummary<'_>> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.format {
      cli::Format::Text => self.value.fmt_text(f),
      cli::Format::JSON => self.value.fmt_json(f),
    }
  }
}

#[derive(Debug, Serialize)]
pub struct Ticket {
  pub id: i32,
  pub state: State,
  pub roles: Option<Vec<String>>,
  pub summary: String,
  pub detail: Option<String>,
  pub data: Option<Vec<u8>>,
  pub owner_id: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl Ticket {
  pub fn init_db(conn: &rusqlite::Connection) -> Result<(), error::Error> {
    conn.execute("
      CREATE TABLE IF NOT EXISTS ticket (
        id         INTEGER PRIMARY KEY,
        state      TEXT NOT NULL,
        roles      TEXT,
        summary    TEXT NOT NULL,
        detail     TEXT,
        data       BLOB,
        owner_id   TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
      )",
      (),
    )?;
    conn.execute("
      CREATE TABLE IF NOT EXISTS ticket_role (
        ticket_id INTEGER NOT NULL REFERENCES ticket (id),
        role      TEXT UNIQUE NOT NULL,
        PRIMARY KEY (ticket_id, role)
      )",
      (),
    )?;
    Ok(())
  }

  pub fn summary(&self) -> TicketSummary<'_> {
    TicketSummary{
      id: self.id,
      state: &self.state,
      roles: &self.roles,
      summary: &self.summary,
      owner_id: &self.owner_id,
      created_at: &self.created_at,
      updated_at: &self.updated_at,
    }
  }

  pub fn formatted<'a>(&'a self, format: &'a cli::Format) -> cli::Formatted<'a, Ticket> {
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

impl Display for cli::Formatted<'_, Ticket> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.format {
      cli::Format::Text => self.value.fmt_text(f),
      cli::Format::JSON => self.value.fmt_json(f),
    }
  }
}

fn from_row(conn: &Connection) -> impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<Ticket> + '_ {
  |row| {
    let id: i32 = row.get(0)?;

    let mut stmt = conn.prepare("SELECT role FROM ticket_role WHERE ticket_id = ?1")?;
    let role_iter = stmt.query_map(rusqlite::params![&id], |row| {
      row.get::<usize, String>(0)
    })?;

    Ok(Ticket {
      id: id,
      state: row.get(1)?,
      summary: row.get(2)?,
      roles: Some(role_iter.collect::<Result<Vec<String>, _>>()?),
      detail: row.get(4)?,
      data: row.get(5)?,
      owner_id: row.get(6)?,
      created_at: row.get(7)?,
      updated_at: row.get(8)?,
    })
  }
}

fn fetch(conn: &Connection, id: i32) -> Result<Ticket, error::Error> {
  let mut stmt = conn.prepare("
    SELECT id, state, summary, roles, detail, data, owner_id, created_at, updated_at
    FROM ticket
    WHERE id = ?1")?;

  Ok(stmt.query_one(rusqlite::params![id], from_row(&conn))?)
}
