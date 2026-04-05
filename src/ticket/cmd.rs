use std::collections::HashSet;

use clap::{Subcommand, Args};
use rusqlite::{Connection, Result};

use crate::error;
use crate::cli;
use crate::ticket::State;
use crate::sqlx;
use crate::ticket;
use crate::Options;

#[derive(Args, Debug)]
pub struct TicketOptions {
  #[clap(long, env="NEXUS_AGENT", help="A unique identifier of the agent operating on the project (use: 'agent new' to assign a new identifier)")]
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
  #[clap(name="get", about="Fetch a ticket")]
  Fetch(FetchTicketOptions),
  #[clap(name="update", about="Update a ticket")]
  Update(UpdateTicketOptions),
  #[clap(name="take", about="Take ownership of a ticket")]
  Take(TakeTicketOptions),
  #[clap(name="abandon", about="Abandon tickets currently owned by an agent that could not be completed so that another agent may take them")]
  Abandon(AbandonTicketOptions),
}

#[derive(Args, Debug)]
pub struct CreateTicketOptions {
  #[clap(long, help="The roles the ticket may be performed by")]
  role: Option<Vec<String>>,
  #[clap(long, help="A brief summary of the ticket")]
  summary: String,
  #[clap(long, help="Read the ticket detail content from the specified file; use '-' for STDIN")]
  detail: Option<String>,
}

#[derive(Args, Debug)]
pub struct ListTicketOptions {
  #[clap(long, help="Only include tickets owned by any of the specified agents")]
  owner: Option<Vec<String>>,
  #[clap(long, help="Only include tickets that can be perfomed by the specified roles")]
  role: Option<Vec<String>>,
  #[clap(long, help="Only include tickets that have one of the specified states")]
  state: Option<Vec<State>>,
  #[clap(long, help="Only include tickets that are assigned to the agent making this request")]
  mine: bool,
  #[clap(long, help="Only include tickets that are available")]
  available: bool,
}

#[derive(Args, Debug)]
pub struct FetchTicketOptions {
  #[clap(long, help="The ticket to fetch")]
  id: i32
}

#[derive(Args, Debug)]
pub struct UpdateTicketOptions {
  #[clap(long, help="The ticket to update")]
  id: i32,
  #[clap(long, help="The state of the ticket")]
  state: Option<State>,
  #[clap(long, help="A brief summary of the note")]
  summary: Option<String>,
  #[clap(long, help="Read the note content from the specified file; use '-' for STDIN")]
  detail: Option<String>,
}

#[derive(Args, Debug)]
pub struct TakeTicketOptions {
  #[clap(long, help="The tickets to take ownership of; specify repeatedly for multiple tickets")]
  id: Vec<i32>,
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
    TicketCommand::Fetch(sub)   => fetch_ticket(opts, ticket, sub, conn),
    TicketCommand::Update(sub)  => update_ticket(opts, ticket, sub, conn),
    TicketCommand::Take(sub)    => take_ticket(opts, ticket, sub, conn),
    TicketCommand::Abandon(sub) => abandon_ticket(opts, ticket, sub, conn),
  }
}

fn ticket_row(conn: &Connection) -> impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<ticket::Ticket> + '_ {
  |row| {
    let id: i32 = row.get(0)?;

    let mut stmt = conn.prepare("SELECT role FROM ticket_role WHERE ticket_id = ?1")?;
    let role_iter = stmt.query_map(rusqlite::params![&id], |row| {
      row.get::<usize, String>(0)
    })?;

    Ok(ticket::Ticket {
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

fn create_ticket(opts: &Options, _ticket: &TicketOptions, create: &CreateTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut val = ticket::Ticket{
    id: 0,
    state: ticket::State::Available,
    summary: create.summary.to_owned(),
    roles: create.role.to_owned(),
    detail: cli::read_input(&create.detail)?,
    data: None,
    owner_id: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  let mut stmt = conn.prepare("
    INSERT INTO ticket (
      state, summary, detail, data, owner_id, created_at, updated_at
    ) VALUES (
      ?1, ?2, ?3, ?4, ?5, ?6, ?7
    )
    RETURNING id"
  )?;

  let mut vals_iter = stmt.query_map(rusqlite::params![
    &val.state, &val.summary, &val.detail, &val.data, &val.owner_id, &val.created_at, &val.updated_at
  ], |row| {
    row.get::<usize, i32>(0)
  })?;
  let val_id = match vals_iter.next() {
    Some(next) => next?,
    None       => return Err(error::Error::ArgumentError("No identifier returned".to_owned())),
  };

  val.id = val_id;

  if let Some(roles) = &val.roles {
    for role in roles {
      conn.execute("
        INSERT INTO ticket_role (
          ticket_id, role
        ) VALUES (
          ?1, ?2
        )",
        (&val_id, &role),
      )?;
    }
  }

  println!("{}", val.formatted(&opts.format()));
  Ok(())
}

fn list_ticket(opts: &Options, ticket: &TicketOptions, list: &ListTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut query = sqlx::Query::new();
  query.push("
    SELECT id, state, summary, roles, detail, data, owner_id, created_at, updated_at
    FROM ticket");

  if let Some(role) = &list.role {
    query
      .push(" INNER JOIN ticket_role ON ticket_role.ticket_id = ticket.id")
      .push_where("ticket_role.role IN ").push_list(role);
  }

  if list.mine  {
    query.push_where("owner_id = ").push_var(ticket.agent.to_owned());
  }else if list.available {
    query
      .push_where("(owner_id IS NULL OR owner_id = ").push_var(ticket.agent.to_owned()).push(")")
      .push_where("state = ").push_var(ticket::State::Available).push(")");
  }

  if let Some(owners) = &list.owner {
    query.push_where("owner_id IN ").push_list(owners);
  }
  if let Some(state) = &list.state {
    query.push_where("state IN ").push_list(state);
  }

  query.push(" ORDER BY updated_at DESC");

  if opts.debug {
    eprintln!("query: {}", &query);
  }

  let mut stmt = conn.prepare(&query.sql)?;
  let vals_iter = stmt.query_map(rusqlite::params_from_iter(query.args), ticket_row(&conn))?;

  let mut n: i32 = 0;
  let format = &opts.format();
  for val in vals_iter {
    let val = val?;
    println!("{}", val.summary().formatted(format));
    n += 1;
  }

  if format == &cli::Format::Text {
    println!("{} found", n);
  }
  Ok(())
}

fn ticket_with_id(_opts: &Options, _ticket: &TicketOptions, id: i32, conn: &Connection) -> Result<ticket::Ticket, error::Error> {
  let mut stmt = conn.prepare("
    SELECT id, state, summary, roles, detail, data, owner_id, created_at, updated_at
    FROM ticket
    WHERE id = ?1")?;

  Ok(stmt.query_one(rusqlite::params![id], ticket_row(&conn))?)
}

fn fetch_ticket(opts: &Options, ticket: &TicketOptions, fetch: &FetchTicketOptions, conn: Connection) -> Result<(), error::Error> {
  println!("{}", ticket_with_id(opts, ticket, fetch.id, &conn)?.formatted(&opts.format()));
  Ok(())
}

fn update_ticket(opts: &Options, ticket: &TicketOptions, update: &UpdateTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut val = ticket_with_id(opts, ticket, update.id, &conn)?;
  val.state = update.state.to_owned().unwrap_or(val.state);
  val.summary = update.summary.to_owned().unwrap_or(val.summary);
  val.detail = cli::read_input(&update.detail)?.or(val.detail);
  val.updated_at = chrono::Utc::now();

  let mut stmt = conn.prepare("
    UPDATE ticket SET state = ?1, summary = ?2, detail = ?3, updated_at = ?4
    WHERE id = ?5"
  )?;
  stmt.execute(rusqlite::params![
    &val.state, &val.summary, &val.detail, &val.updated_at,
    update.id,
  ])?;

  println!("{}", val.formatted(&opts.format()));
  Ok(())
}

fn take_ticket(opts: &Options, ticket: &TicketOptions, take: &TakeTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut query = sqlx::Query::new();
  query
    .push("UPDATE ticket SET owner_id = ").push_var(ticket.agent.to_owned())
    .push_where("id IN ").push_list(&take.id)
    .push_where("(owner_id IS NULL OR owner_id = ").push_var(ticket.agent.to_owned()).push(" OR ").push_var(take.force).push(" = TRUE)")
    .push(" RETURNING id, state, summary, roles, detail, data, owner_id, created_at, updated_at");

  if opts.debug {
    eprintln!("query: {}", &query);
  }

  let mut stmt = conn.prepare(&query.sql)?;
  let vals_iter = stmt.query_map(rusqlite::params_from_iter(query.args), ticket_row(&conn))?;

  let mut taken = Vec::new();
  for val in vals_iter {
    let val = val?;
    taken.push(val.id);
    match &opts.format() {
      cli::Format::JSON => println!("{}", cli::Formatted::from(&val, &cli::Format::JSON)),
      cli::Format::Text => println!("{} owns ticket {}", ticket.agent, ticket::format_ids(&take.id)),
    }
  };

  let take_set: HashSet<_> = (&take.id).into_iter().collect();
  let took_set: HashSet<_> = (&taken).into_iter().collect();
  let missing: Vec<&i32> = take_set.difference(&took_set).cloned().collect();

  if !missing.is_empty() {
    eprintln!("warning: could not take: {}", ticket::format_ids(&missing))
  }
  Ok(())
}

fn abandon_ticket(opts: &Options, ticket: &TicketOptions, abandon: &AbandonTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut query = sqlx::Query::new();
  query
    .push("UPDATE ticket SET owner_id = NULL")
    .push_where("owner_id = ").push_var(ticket.agent.to_owned())
    .push_where("state NOT IN ").push_list(&[State::Done]);

  if let Some(ids) = &abandon.id {
    query.push_where("id IN ").push_list(ids);
  }

  query.push(" RETURNING id, state, summary, roles, detail, data, owner_id, created_at, updated_at");

  if opts.debug {
    eprintln!("query: {}", &query);
  }

  let mut stmt = conn.prepare(&query.sql)?;
  let vals_iter = stmt.query_map(rusqlite::params_from_iter(query.args), ticket_row(&conn))?;
  for val in vals_iter {
    let val = val?;
    match &opts.format() {
      cli::Format::JSON => println!("{}", cli::Formatted::from(&val, &cli::Format::JSON)),
      cli::Format::Text => println!("{} is abandoned", ticket::format_id(val.id)),
    }
  }

  Ok(())
}
