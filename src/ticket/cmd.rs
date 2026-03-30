use std::prelude::rust_2015;

use clap::{Subcommand, Args};
use rusqlite::{Connection, Result};

use crate::error;
use crate::ticket;
use crate::Options;

#[derive(Args, Debug)]
pub struct TicketOptions {
  #[clap(subcommand)]
  command: TicketCommand,
}

#[derive(Subcommand, Debug)]
pub enum TicketCommand {
  #[clap(name="new", about="Create a ticket")]
  Create(CreateTicketOptions),
  #[clap(name="list", about="List tickets")]
  List(ListTicketOptions),
  #[clap(name="take", about="Take ownership of a ticket")]
  Take(TakeTicketOptions),
  #[clap(name="abandon", about="Abandon tickets owned by an agent")]
  Abandon(AbandonTicketOptions),
}

#[derive(Args, Debug)]
pub struct CreateTicketOptions {
  #[clap(long, help="A brief summary of the ticket")]
  summary: String,
}

#[derive(Args, Debug)]
pub struct ListTicketOptions {
  #[clap(long, help="Only include issues owned by any of the specified agents")]
  owner: Option<Vec<String>>,
}

#[derive(Args, Debug)]
pub struct TakeTicketOptions {
  #[clap(long, help="The ticket to take ownership of")]
  id: i32,
  #[clap(long, help="Take ownership even if the ticket is already owned by another agent (this should only be used to resolve proven coordination failures)")]
  force: bool
}

#[derive(Args, Debug)]
pub struct AbandonTicketOptions {
  #[clap(long, help="The tickets to abandon ownership of or abandon all tickets owned by the agent if none are specified")]
  id: Vec<i32>,
}

pub fn ticket(opts: &Options, ticket: &TicketOptions, conn: Connection) -> Result<(), error::Error> {
  match &ticket.command {
    TicketCommand::Create(sub)  => create_ticket(opts, ticket, sub, conn),
    TicketCommand::List(sub)    => list_ticket(opts, ticket, sub, conn),
    TicketCommand::Take(sub)    => take_ticket(opts, ticket, sub, conn),
    TicketCommand::Abandon(sub) => abandon_ticket(opts, ticket, sub, conn),
  }
}

fn create_ticket(_opts: &Options, _ticket: &TicketOptions, create: &CreateTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let tkt = ticket::Ticket{
    id: 0,
    state: ticket::State::Available,
    summary: create.summary.to_owned(),
    detail: None,
    data: None,
    owner_id: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  conn.execute("
    INSERT INTO ticket (
      state, summary, detail, data, created_at, updated_at
    ) VALUES (
      ?1, ?2, ?3, ?4, ?5, ?6
    )",
    (&tkt.state, &tkt.summary, &tkt.detail, &tkt.data, &tkt.created_at, &tkt.updated_at),
  )?;

  Ok(())
}

fn list_ticket(_opts: &Options, _ticket: &TicketOptions, list: &ListTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut query = "
    SELECT id, state, summary, detail, data, owner_id, created_at, updated_at
    FROM ticket".to_string();

  let mut args: Vec<&dyn rusqlite::types::ToSql> = vec![];
  if let Some(owners) = &list.owner {
    let mut n = 0;
    query.push_str("
      WHERE owner_id IN (");
    for owner in owners {
      if n > 0 {
        query.push_str(", ");
      }
      query.push_str(&format!("?{}", args.len() + 1));
      args.push(owner);
      n = n + 1;
    }
    query.push_str(")");
  }

  let mut stmt = conn.prepare(&query)?;
  let tkts_iter = stmt.query_map(rusqlite::params_from_iter(args.iter()), |row| {
    Ok(ticket::Ticket {
      id: row.get(0)?,
      state: row.get(1)?,
      summary: row.get(2)?,
      detail: row.get(3)?,
      data: row.get(4)?,
      owner_id: row.get(5)?,
      created_at: row.get(6)?,
      updated_at: row.get(7)?,
    })
  })?;

  for tkt in tkts_iter {
    let tkt = tkt?;
    println!("#{} {:?}", tkt.id, tkt);
  }
  Ok(())
}

fn take_ticket(opts: &Options, _ticket: &TicketOptions, take: &TakeTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let agent_id = match &opts.agent {
    Some(id) => id,
    None     => return Err(error::Error::ArgumentError("Agent identifier is required".to_owned())),
  };

  let mut stmt = conn.prepare("
    UPDATE ticket SET owner_id = ?1
    WHERE id = ?2
    AND (owner_id IS NULL OR owner_id = ?3 OR ?4 = TRUE)
    RETURNING id",
  )?;

  let mut tkts_iter = stmt.query_map(rusqlite::params![
    agent_id, take.id, agent_id, take.force,
  ], |row| {
    Ok(row.get::<usize, i32>(0))
  })?;

  if !match tkts_iter.next() {
    Some(next) => next?? == take.id,
    None       => false,
  } {
    return Err(error::Error::ArgumentError(format!("{} is already taken by {}", ticket::format_id(take.id), agent_id).to_owned()));
  }

  println!("{} owns {}", agent_id, ticket::format_id(take.id));
  Ok(())
}

fn abandon_ticket(_opts: &Options, _ticket: &TicketOptions, abandon: &AbandonTicketOptions, conn: Connection) -> Result<(), error::Error> {
  Ok(())
}
