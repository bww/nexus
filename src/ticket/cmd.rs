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
}

#[derive(Args, Debug)]
pub struct CreateTicketOptions {
  #[clap(long, help="A brief summary of the ticket")]
  summary: String,
}

#[derive(Args, Debug)]
pub struct ListTicketOptions {
}

#[derive(Args, Debug)]
pub struct TakeTicketOptions {
  #[clap(long, help="The ticket to take ownership of")]
  id: i32,
}

pub fn ticket(opts: &Options, ticket: &TicketOptions, conn: Connection) -> Result<(), error::Error> {
  match &ticket.command {
    TicketCommand::Create(sub) => create_ticket(opts, ticket, sub, conn),
    TicketCommand::List(sub)   => list_ticket(opts, ticket, sub, conn),
    TicketCommand::Take(sub)   => take_ticket(opts, ticket, sub, conn),
  }
}

fn create_ticket(_opts: &Options, _ticket: &TicketOptions, create: &CreateTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let tkt = ticket::Ticket{
    id: 0,
    state: ticket::State::Available,
    summary: create.summary.to_owned(),
    detail: None,
    data: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  conn.execute(
    "INSERT INTO ticket (
      state, summary, detail, data, created_at, updated_at
    ) VALUES (
      ?1, ?2, ?3, ?4, ?5, ?6
    )",
    (&tkt.state, &tkt.summary, &tkt.detail, &tkt.data, &tkt.created_at, &tkt.updated_at),
  )?;

  Ok(())
}

fn list_ticket(_opts: &Options, _ticket: &TicketOptions, _list: &ListTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut stmt = conn.prepare(
    "SELECT id, state, summary, detail, data, created_at, updated_at
    FROM ticket"
  )?;

  let tkts_iter = stmt.query_map([], |row| {
    Ok(ticket::Ticket {
      id: row.get(0)?,
      state: row.get(1)?,
      summary: row.get(2)?,
      detail: row.get(3)?,
      data: row.get(4)?,
      created_at: row.get(5)?,
      updated_at: row.get(6)?,
    })
  })?;

  for tkt in tkts_iter {
    let tkt = tkt.unwrap();
    println!("#{} {:?}", tkt.id, tkt);
  }
  Ok(())
}

fn take_ticket(_opts: &Options, _ticket: &TicketOptions, take: &TakeTicketOptions, conn: Connection) -> Result<(), error::Error> {
  Ok(())
}
