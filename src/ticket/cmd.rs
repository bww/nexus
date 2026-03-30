use clap::{Subcommand, Args};
use rusqlite::{Connection, Result};

use crate::error;
use crate::{sql_index, sql_where, sql_list};
use crate::ticket;
use crate::Options;

#[derive(Args, Debug)]
pub struct TicketOptions {
  #[clap(long, help="A unique identifier of the agent operating on the project (use: 'agent new' to assign a new identifier)")]
  agent: String,
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
  #[clap(long, help="Only include issues that are available")]
  available: bool,
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
  id: Option<Vec<i32>>,
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

fn list_ticket(_opts: &Options, ticket: &TicketOptions, list: &ListTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut query = "
    SELECT id, state, summary, detail, data, owner_id, created_at, updated_at
    FROM ticket".to_string();

  let mut args: Vec<&dyn rusqlite::types::ToSql> = vec![];

  if list.available {
    sql_where!(query, args);
    query.push_str(&format!("(owner_id IS NULL OR owner_id = ?{})", sql_index!(args)));
    args.push(&ticket.agent);
    query.push_str(&format!(" AND state = ?{}", sql_index!(args)));
    args.push(&ticket::State::Available);
  }

  if let Some(owners) = &list.owner {
    sql_where!(query, args);
    query.push_str("owner_id IN (");
    query.push_str("owner_id IN ");
    sql_list!(query, args, owners);
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
    println!("{} {:?}", ticket::format_id(tkt.id), tkt);
  }
  Ok(())
}

fn take_ticket(_opts: &Options, ticket: &TicketOptions, take: &TakeTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut stmt = conn.prepare("
    UPDATE ticket SET owner_id = ?1
    WHERE id = ?2
    AND (owner_id IS NULL OR owner_id = ?3 OR ?4 = TRUE)
    RETURNING id",
  )?;

  let mut tkts_iter = stmt.query_map(rusqlite::params![
    ticket.agent, take.id, ticket.agent, take.force,
  ], |row| {
    Ok(row.get::<usize, i32>(0))
  })?;

  if !match tkts_iter.next() {
    Some(next) => next?? == take.id,
    None       => false,
  } {
    return Err(error::Error::ArgumentError(format!("Ticket {} is already taken by {}", ticket::format_id(take.id), ticket.agent).to_owned()));
  }

  println!("{} owns ticket {}", ticket.agent, ticket::format_id(take.id));
  Ok(())
}

fn abandon_ticket(_opts: &Options, ticket: &TicketOptions, abandon: &AbandonTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut query = "
    UPDATE ticket SET owner_id = NULL
    WHERE owner_id = ?1
    AND state IN (?2, ?3)".to_string();

  let mut args: Vec<&dyn rusqlite::types::ToSql> = vec![
    &ticket.agent,
     // only incomplete tickets are abandoned
    &ticket::State::Available,
    &ticket::State::InProgress,
  ];

  if let Some(ids) = &abandon.id {
    sql_where!(query, args);
    query.push_str("id IN ");
    sql_list!(query, args, ids);
  }

  query.push_str("
    RETURNING id");

  let mut stmt = conn.prepare(&query)?;
  let tkts_iter = stmt.query_map(rusqlite::params_from_iter(args.iter()), |row| {
    Ok(row.get::<usize, i32>(0))
  })?;

  for tkt in tkts_iter {
    let tkt = tkt??;
    println!("{} is abandoned", ticket::format_id(tkt));
  }

  Ok(())
}
