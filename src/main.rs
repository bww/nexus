use std::{fs, path, process};

use colored::Colorize;
use clap::{Parser, Subcommand};
use rusqlite::{Connection, Result};

mod error;
mod sqlx;
mod agent;
mod ticket;
mod note;

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
  #[clap(name="agent", about="Agents")]
  Agent(agent::cmd::AgentOptions),
  #[clap(name="ticket", about="Tickets")]
  Ticket(ticket::cmd::TicketOptions),
}

fn main() {
  match cmd(){
    Ok(_)    => return,
    Err(err) => {
      eprintln!("{}", &format!("error: {}", err).yellow().bold());
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
  ticket::Ticket::init_db(&conn)?;

  match &opts.command {
    Command::Agent(sub)  => agent::cmd::agent(&opts, sub, conn),
    Command::Ticket(sub) => ticket::cmd::ticket(&opts, sub, conn),
  }
}
