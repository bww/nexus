use std::{fs, path, process};

use colored::Colorize;
use clap::{Parser, Subcommand, Args, CommandFactory};
use rusqlite::{Connection, Result};
use gix;

mod error;
mod cli;
mod sqlx;
mod agent;
mod ticket;
mod note;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Options {
  #[clap(long, env="NEXUS_DEBUG", help="Enable debugging mode. When debugging is enabled, additional output is written to aid in debugging.")]
  debug: bool,
  #[clap(long, env="NEXUS_VERBOSE", help="Enable verbose output. When verbose mode is enabled, some data that is normally not displayed is included.")]
  verbose: bool,
  #[clap(long, env="NEXUS_PROJECT", help="Path to the project root.")]
  project: Option<String>,
  #[clap(long, env="NEXUS_DATABASE", help="Path to the Nexus database. (Prefer the --project flag unless you have a specific reason for using --database)")]
  database: Option<String>,
  #[clap(long, env="NEXUS_FORMAT", help="The output format to use. The JSON output format is the most complete.")]
  format: Option<cli::Format>,
  #[clap(long, env="NEXUS_AGENT", global=true, help="A unique identifier of the agent operating on the project; required for all subcommands except `agent` (use: 'agent new' to assign a new identifier)")]
  agent: Option<String>,
  #[clap(subcommand)]
  command: Command,
}

impl Options {
  pub fn format(&self) -> cli::Format {
    self.format.as_ref().unwrap_or(&cli::Format::JSON).to_owned()
  }

  pub fn agent(&self) -> Result<&str, error::Error> {
    self.agent.as_deref().ok_or_else(||
      error::Error::ArgumentError("--agent (or NEXUS_AGENT) is required".to_owned()))
  }

  pub fn project(&self) -> Result<path::PathBuf, error::Error> {
    let proj = match &self.project {
      Some(proj) => path::PathBuf::from(proj),
      None       => Self::discover_project(".")?,
    };
    if self.verbose {
      eprintln!("project: {}", &proj.to_string_lossy());
    }
    Ok(proj)
  }

  fn discover_project(rel: &str) -> Result<path::PathBuf, error::Error> {
    let mut path = gix::discover(rel)?.path().to_path_buf();
    path.pop();          // drop '.git'
    path.push(".nexus"); // this may not exist
    Ok(path)
  }
}

#[derive(Subcommand, Debug)]
enum Command {
  #[clap(name="ticket", about="Manage tickets")]
  Ticket(ticket::cmd::TicketOptions),
  #[clap(name="note", about="Manage notes")]
  Note(note::cmd::NoteOptions),
  #[clap(name="agent", about="Agent context utilites")]
  Agent(agent::cmd::AgentOptions),
  #[clap(name="docs", about="Display documentation about Nexus")]
  Docs(DocsOptions),
}

#[derive(Args, Debug)]
pub struct DocsOptions {}

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

  if !matches!(opts.command, Command::Agent(_)) && opts.agent.is_none() {
    Options::command().error(
      clap::error::ErrorKind::MissingRequiredArgument,
      "--agent (or NEXUS_AGENT) is required for this subcommand",
    ).exit();
  }

  let dbpath = match &opts.database {
    Some(path) => path::PathBuf::from(path),
    None       => {
      let mut proj = opts.project()?;
      proj.push(".nexus/data.db");
      proj
    },
  };

  if let Some(dir) = dbpath.parent() {
    fs::create_dir_all(dir)?;
  }

  let conn = Connection::open(&dbpath)?;
  ticket::Ticket::init_db(&conn)?;
  note::Note::init_db(&conn)?;

  match &opts.command {
    Command::Agent(sub)  => agent::cmd::agent(&opts, sub, conn),
    Command::Ticket(sub) => ticket::cmd::ticket(&opts, sub, conn),
    Command::Note(sub)   => note::cmd::note(&opts, sub, conn),
    Command::Docs(sub)   => docs(&opts, sub),
  }
}

fn docs(_opts: &Options, _docs: &DocsOptions) -> Result<(), error::Error> {
  println!("{}", clap_markdown::help_markdown::<Options>());
  Ok(())
}
