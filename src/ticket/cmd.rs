use clap::{Subcommand, Args};
use rusqlite::{Connection, Result};

use crate::error;
use crate::cli;
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
  #[clap(name="get", about="Fetch a ticket")]
  Fetch(FetchTicketOptions),
  #[clap(name="take", about="Take ownership of a ticket")]
  Take(TakeTicketOptions),
  #[clap(name="abandon", about="Abandon tickets owned by an agent")]
  Abandon(AbandonTicketOptions),
}

#[derive(Args, Debug)]
pub struct CreateTicketOptions {
  #[clap(long, help="The roles the ticket may be performed by")]
  role: Option<Vec<String>>,
  #[clap(long, help="A brief summary of the ticket")]
  summary: String,
}

#[derive(Args, Debug)]
pub struct ListTicketOptions {
  #[clap(long, help="Only include issues owned by any of the specified agents")]
  owner: Option<Vec<String>>,
  #[clap(long, help="Only include issues that can be perfomed by the specified roles")]
  role: Option<Vec<String>>,
  #[clap(long, help="Only include issues that are assigned to the agent making this request")]
  mine: bool,
  #[clap(long, help="Only include issues that are available")]
  available: bool,
}

#[derive(Args, Debug)]
pub struct FetchTicketOptions {
  #[clap(long, help="The ticket to fetch")]
  id: i32
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
    TicketCommand::Fetch(sub)   => fetch_ticket(opts, ticket, sub, conn),
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

fn create_ticket(_opts: &Options, _ticket: &TicketOptions, create: &CreateTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let tkt = ticket::Ticket{
    id: 0,
    state: ticket::State::Available,
    summary: create.summary.to_owned(),
    roles: create.role.to_owned(),
    detail: None,
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

  let mut tkts_iter = stmt.query_map(rusqlite::params![
    &tkt.state, &tkt.summary, &tkt.detail, &tkt.data, &tkt.owner_id, &tkt.created_at, &tkt.updated_at
  ], |row| {
    row.get::<usize, i32>(0)
  })?;
  let tkt_id = match tkts_iter.next() {
    Some(next) => next?,
    None       => return Err(error::Error::ArgumentError("No identifier returned".to_owned())),
  };

  if let Some(roles) = tkt.roles {
    for role in roles {
      conn.execute("
        INSERT INTO ticket_role (
          ticket_id, role
        ) VALUES (
          ?1, ?2
        )",
        (&tkt_id, &role),
      )?;
    }
  }

  Ok(())
}

fn list_ticket(opts: &Options, ticket: &TicketOptions, list: &ListTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut query = "
    SELECT id, state, summary, roles, detail, data, owner_id, created_at, updated_at
    FROM ticket".to_string();

  let mut args: Vec<&dyn rusqlite::types::ToSql> = vec![];

  if let Some(role) = &list.role {
    query.push_str("
      INNER JOIN ticket_role
      ON ticket_role.ticket_id = ticket.id"
    );
    sql_where!(query, args);
    query.push_str("ticket_role.role IN ");
    sql_list!(query, args, role);
  }

  if list.mine  {
    sql_where!(query, args);
    query.push_str(&format!("owner_id = ?{}", sql_index!(args)));
    args.push(&ticket.agent);
  }else if list.available {
    sql_where!(query, args);
    query.push_str(&format!("(owner_id IS NULL OR owner_id = ?{})", sql_index!(args)));
    args.push(&ticket.agent);
    sql_where!(query, args);
    query.push_str(&format!("state = ?{}", sql_index!(args)));
    args.push(&ticket::State::Available);
  }

  if let Some(owners) = &list.owner {
    sql_where!(query, args);
    query.push_str("owner_id IN ");
    sql_list!(query, args, owners);
  }

  let mut stmt = conn.prepare(&query)?;
  let tkts_iter = stmt.query_map(rusqlite::params_from_iter(args.iter()), ticket_row(&conn))?;

  let mut n: i32 = 0;
  let format = &opts.format();
  for tkt in tkts_iter {
    let tkt = tkt?;
    println!("{}", tkt.formatted(format));
    n += 1;
  }

  if format == &cli::Format::Text {
    println!("{} found", n);
  }
  Ok(())
}

fn fetch_ticket(opts: &Options, _ticket: &TicketOptions, fetch: &FetchTicketOptions, conn: Connection) -> Result<(), error::Error> {
  let mut stmt = conn.prepare("
    SELECT id, state, summary, roles, detail, data, owner_id, created_at, updated_at
    FROM ticket
    WHERE id = ?1")?;

  let tkt = stmt.query_one(rusqlite::params![fetch.id], ticket_row(&conn))?;

  println!("{}", tkt.formatted(&opts.format()));
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
    row.get::<usize, i32>(0)
  })?;

  if !match tkts_iter.next() {
    Some(next) => next? == take.id,
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
    row.get::<usize, i32>(0)
  })?;

  for tkt in tkts_iter {
    let tkt = tkt?;
    println!("{} is abandoned", ticket::format_id(tkt));
  }

  Ok(())
}
