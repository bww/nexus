use std::{fs, path, process};
use std::fmt::{self, Display};
use std::str::FromStr;

use colored::Colorize;
use clap::{Parser, Subcommand, Args};
use rusqlite::{Connection, Result};

mod error;

#[derive(Debug)]
enum State {
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
  fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>> {
    Ok(rusqlite::types::ToSqlOutput::Owned(
      rusqlite::types::Value::Text(self.to_string())
    ))
  }
}

#[derive(Debug)]
struct Ticket {
  id: i32,
  name: String,
  state: State,
  data: Option<Vec<u8>>,
}

#[derive(Debug)]
struct Note {
  id: i32,
  name: String,
  data: Option<Vec<u8>>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Options {
  #[clap(long, help="Enable debugging mode")]
  debug: bool,
  #[clap(long, help="Enable verbose output")]
  verbose: bool,
  #[clap(long, help="Path to the project root")]
  project: String,
  #[clap(long, help="Path to the database")]
  database: Option<String>,
  #[clap(subcommand)]
  command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
  #[clap(name="ticket", about="Tickets")]
  Ticket(TicketOptions),
}

#[derive(Args, Debug)]
struct TicketOptions {
  #[clap(subcommand)]
  command: TicketCommand,
}

#[derive(Subcommand, Debug)]
enum TicketCommand {
  #[clap(name="list", about="List tickets")]
  List(ListTicketOptions),
}

#[derive(Args, Debug)]
struct ListTicketOptions {
}

fn main() {
  match cmd(){
    Ok(_)    => return,
    Err(err) => {
      eprintln!("{}", &format!("* * * {}", err).yellow().bold());
      process::exit(1);
    },
  };
}

fn cmd() -> Result<(), error::Error> {
  let opts = Options::parse();

  let dbpath = match &opts.database {
    Some(path) => path::PathBuf::from(path),
    None       => [&opts.project, ".nexus/data.db"].iter().collect(),
  };

  if let Some(dir) = dbpath.parent() {
    fs::create_dir_all(dir)?;
  }

  let conn = Connection::open(&dbpath)?;
  conn.execute(
    "CREATE TABLE IF NOT EXISTS ticket (
      id    INTEGER PRIMARY KEY,
      name  TEXT NOT NULL,
      state TEXT NOT NULL,
      data  BLOB
    )",
    (),
  )?;

  match &opts.command {
    Command::Ticket(sub) => ticket(&opts, sub, conn),
  }
}

fn ticket(opts: &Options, ticket: &TicketOptions, conn: Connection) -> Result<(), error::Error> {
  match &ticket.command {
    TicketCommand::List(sub) => list_ticket(opts, ticket, sub, conn),
  }
}

fn list_ticket(_opts: &Options, _ticket: &TicketOptions, _list: &ListTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let tkt = Ticket {
    id: 0,
    name: "Jambo".to_string(),
    state: State::Available,
    data: None,
  };
  conn.execute(
    "INSERT INTO ticket (name, state, data) VALUES (?1, ?2, ?3)",
    (&tkt.name, &tkt.state, &tkt.data),
  )?;

  let mut stmt = conn.prepare("SELECT id, name, state, data FROM ticket")?;
  let person_iter = stmt.query_map([], |row| {
    Ok(Ticket {
      id: row.get(0)?,
      name: row.get(1)?,
      state: row.get(2)?,
      data: row.get(3)?,
    })
  })?;

  for person in person_iter {
    println!("Found Ticket {:?}", person.unwrap());
  }
  Ok(())
}
