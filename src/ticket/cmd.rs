use clap::{Subcommand, Args};
use rusqlite::{Connection, Result};
use chrono::{DateTime, Local};

use crate::error;
use crate::cli;
use crate::ticket::State;
use crate::sqlx;
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
  #[clap(name="get", about="Fetch tickets by identifier")]
  Fetch(FetchTicketOptions),
  #[clap(name="update", about="Update a ticket")]
  Update(UpdateTicketOptions),
  #[clap(name="take", about="Take ownership of a ticket")]
  Take(TakeTicketOptions),
  #[clap(name="abandon", about="Abandon tickets owned by an agent so that another agent may take them")]
  Abandon(AbandonTicketOptions),
}

#[derive(Args, Debug)]
pub struct CreateTicketOptions {
  #[clap(long, help="The agent roles the ticket may be performed by")]
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
  #[clap(long, help="Only include tickets created before the specified date and time")]
  created_before: Option<DateTime<Local>>,
  #[clap(long, help="Only include tickets created after the specified date and time")]
  created_after: Option<DateTime<Local>>,
  #[clap(long, help="Only include tickets updated before the specified date and time")]
  updated_before: Option<DateTime<Local>>,
  #[clap(long, help="Only include tickets updated after the specified date and time")]
  updated_after: Option<DateTime<Local>>,
}

#[derive(Args, Debug)]
pub struct FetchTicketOptions {
  #[clap(long, help="The ticket to fetch")]
  id: Vec<i32>
}

#[derive(Args, Debug)]
pub struct UpdateTicketOptions {
  #[clap(long, help="The ticket to update")]
  id: i32,
  #[clap(long, help="The fencing token for resolving concurrent updates")]
  fence: i32,
  #[clap(long, help="The roles the ticket may be performed by")]
  role: Option<Vec<String>>,
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

fn create_ticket(opts: &Options, _ticket: &TicketOptions, create: &CreateTicketOptions, mut conn: Connection) -> Result<(), error::Error> {
  let mut val = ticket::Ticket{
    id: 0,
    fence: 0,
    state: ticket::State::Available,
    summary: create.summary.to_owned(),
    roles: create.role.to_owned(),
    detail: cli::read_input(&create.detail)?,
    data: None,
    owner_id: None,
    references: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  let tx = conn.transaction()?;

  {
    let mut stmt = tx.prepare("
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
  }

  if let Some(roles) = &val.roles {
    for role in roles {
      tx.execute("
        INSERT INTO ticket_role (
          ticket_id, role
        ) VALUES (
          ?1, ?2
        )",
        rusqlite::params![&val.id, &role],
      )?;
    }
  }

  tx.commit()?;
  println!("{}", val.formatted(&opts.format()));
  Ok(())
}

fn list_ticket(opts: &Options, _ticket: &TicketOptions, list: &ListTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut query = sqlx::Query::new();
  query.push("
    SELECT id, fence, state, summary, detail, data, owner_id, created_at, updated_at
    FROM ticket");

  if let Some(role) = &list.role {
    query
      .push(" INNER JOIN ticket_role ON ticket_role.ticket_id = ticket.id")
      .push_where("ticket_role.role IN ").push_list(role);
  }

  if list.mine && list.available {
    query
      .push_where("owner_id = ").push_var(opts.agent()?.to_owned())
      .push_where("state = ").push_var(ticket::State::Available);
  }else if list.mine {
    query.push_where("owner_id = ").push_var(opts.agent()?.to_owned());
  }else if list.available {
    query
      .push_where("(owner_id IS NULL OR owner_id = ").push_var(opts.agent()?.to_owned()).push(")")
      .push_where("state = ").push_var(ticket::State::Available);
  }

  if let Some(owners) = &list.owner {
    query.push_where("owner_id IN ").push_list(owners);
  }
  if let Some(state) = &list.state {
    query.push_where("state IN ").push_list(state);
  }

  if let Some(created_before) = &list.created_before {
    query.push_where("created_at < ").push_var(created_before.to_owned());
  }
  if let Some(updated_before) = &list.updated_before {
    query.push_where("updated_at < ").push_var(updated_before.to_owned());
  }
  if let Some(created_after) = &list.created_after {
    query.push_where("created_at > ").push_var(created_after.to_owned());
  }
  if let Some(updated_after) = &list.updated_after {
    query.push_where("updated_at > ").push_var(updated_after.to_owned());
  }

  query.push(" ORDER BY updated_at DESC");

  if opts.debug {
    eprintln!("query: {}", &query);
  }

  let mut stmt = conn.prepare(&query.sql)?;
  let vals_iter = stmt.query_map(rusqlite::params_from_iter(query.args), ticket::from_row(&conn))?;

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

fn fetch_ticket(opts: &Options, _ticket: &TicketOptions, fetch: &FetchTicketOptions, conn: Connection) -> Result<(), error::Error> {
  for id in &fetch.id {
    println!("{}", ticket::fetch(&conn, *id)?.formatted(&opts.format()));
  }
  Ok(())
}

fn update_ticket(opts: &Options, _ticket: &TicketOptions, update: &UpdateTicketOptions, mut conn: Connection) -> Result<(), error::Error> {
  let mut val = ticket::fetch(&conn, update.id)?;
  val.fence = update.fence + 1; // increment the fence on update
  val.roles = update.role.to_owned().or(val.roles);
  val.state = update.state.to_owned().unwrap_or(val.state);
  val.summary = update.summary.to_owned().unwrap_or(val.summary);
  val.detail = cli::read_input(&update.detail)?.or(val.detail);
  val.updated_at = chrono::Utc::now();

  let tx = conn.transaction()?;

  let mut query = sqlx::Query::new();
  query
    .push("UPDATE ticket SET")
    .push("  fence = ").push_var(val.fence.to_owned())
    .push(", state = ").push_var(val.state.to_owned())
    .push(", summary = ").push_var(val.summary.to_owned())
    .push(", detail = ").push_var(val.detail.to_owned())
    .push(", updated_at = ").push_var(val.updated_at);

  query
    .push_where("id = ").push_var(update.id)
    .push_where("fence = ").push_var(update.fence)
    .push_where("(owner_id IS NULL OR owner_id = ").push_var(opts.agent()?.to_owned()).push(")");

  query.push(" RETURNING id");

  if opts.debug {
    eprintln!("query: {}", &query);
  }

  let updated: Vec<i32> = tx
    .prepare(&query.sql)?
    .query_map(rusqlite::params_from_iter(query.args), sqlx::results::id_from_row(&tx))?
    .collect::<rusqlite::Result<Vec<_>>>()?;

  if let Some(roles) = &update.role {
    for id in &updated {
      tx.execute("
        DELETE FROM ticket_role
        WHERE ticket_id = ?1",
        rusqlite::params![&id],
      )?;
      for role in roles {
        tx.execute("
          INSERT INTO ticket_role (
            ticket_id, role
          ) VALUES (
            ?1, ?2
          )",
          rusqlite::params![&id, &role],
        )?;
      }
    }
  }

  tx.commit()?;

  if let Some(missing) = sqlx::results::missing_one(update.id, &updated) {
    return Err(error::Error::CommandError(format!("error: could not update: {}", ticket::format_id(missing))));
  }

  println!("{}", val.formatted(&opts.format()));
  Ok(())
}

fn take_ticket(opts: &Options, _ticket: &TicketOptions, take: &TakeTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let agent = opts.agent()?.to_owned();
  let mut query = sqlx::Query::new();
  query
    .push("UPDATE ticket SET")
    .push("  owner_id = ").push_var(agent.to_owned())
    .push(", fence = fence + 1")
    .push_where("id IN ").push_list(&take.id)
    .push_where("(owner_id IS NULL OR owner_id = ").push_var(agent.to_owned()).push(" OR ").push_var(take.force).push(" = TRUE)")
    .push(" RETURNING id, fence, state, summary, detail, data, owner_id, created_at, updated_at");

  if opts.debug {
    eprintln!("query: {}", &query);
  }

  let mut stmt = conn.prepare(&query.sql)?;
  let vals_iter = stmt.query_map(rusqlite::params_from_iter(query.args), ticket::from_row(&conn))?;

  let mut taken = Vec::new();
  for val in vals_iter {
    let val = val?;
    taken.push(val.id);
    match &opts.format() {
      cli::Format::JSON => println!("{}", cli::Formatted::from(&val, &cli::Format::JSON)),
      cli::Format::Text => println!("{} owns ticket {}", &agent, ticket::format_ids(&take.id)),
    }
  };

  let missing: Vec<&i32> = sqlx::results::missing(&take.id, &taken);
  if !missing.is_empty() {
    return Err(error::Error::CommandError(format!("error: could not take: {}", ticket::format_ids(&missing))));
  }
  Ok(())
}

fn abandon_ticket(opts: &Options, _ticket: &TicketOptions, abandon: &AbandonTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut query = sqlx::Query::new();
  query
    .push("UPDATE ticket SET")
    .push("  owner_id = NULL")
    .push(", fence = fence + 1")
    .push_where("owner_id = ").push_var(opts.agent()?.to_owned())
    .push_where("state NOT IN ").push_list(&[State::Done]);

  if let Some(ids) = &abandon.id {
    query.push_where("id IN ").push_list(ids);
  }

  query.push(" RETURNING id, fence, state, summary, detail, data, owner_id, created_at, updated_at");

  if opts.debug {
    eprintln!("query: {}", &query);
  }

  let mut stmt = conn.prepare(&query.sql)?;
  let vals_iter = stmt.query_map(rusqlite::params_from_iter(query.args), ticket::from_row(&conn))?;
  for val in vals_iter {
    let val = val?;
    match &opts.format() {
      cli::Format::JSON => println!("{}", cli::Formatted::from(&val, &cli::Format::JSON)),
      cli::Format::Text => println!("{} is abandoned", ticket::format_id(val.id)),
    }
  }

  Ok(())
}
