use clap::{Subcommand, Args};
use rusqlite::{Connection, Result};
use chrono::{DateTime, Local};

use crate::error;
use crate::cli;
use crate::sqlx;
use crate::note;
use crate::Options;

#[derive(Args, Debug)]
pub struct NoteOptions {
  #[clap(subcommand)]
  command: NoteCommand,
}

#[derive(Subcommand, Debug)]
pub enum NoteCommand {
  #[clap(name="new", about="Create a note")]
  Create(CreateNoteOptions),
  #[clap(name="list", about="List notes")]
  List(ListNoteOptions),
  #[clap(name="get", about="Fetch a note")]
  Fetch(FetchNoteOptions),
  #[clap(name="update", about="Update a note")]
  Update(UpdateNoteOptions),
  #[clap(name="delete", about="Delete a note")]
  Delete(DeleteNoteOptions),
}

#[derive(Args, Debug)]
pub struct CreateNoteOptions {
  #[clap(long, help="A brief summary of the note")]
  summary: String,
  #[clap(long, help="Read the note content from the specified file; use '-' for STDIN")]
  detail: Option<String>,
  #[clap(long, help="The commit SHA the note refers to")]
  commit: Option<String>,
}

#[derive(Args, Debug)]
pub struct ListNoteOptions {
  #[clap(long, help="Only include notes created by any of the specified agents")]
  creator: Option<Vec<String>>,
  #[clap(long, help="Only include notes associated with any of the specified commit SHA")]
  commit: Option<Vec<String>>,
  #[clap(long, help="Only include notes created before the specified date and time")]
  created_before: Option<DateTime<Local>>,
  #[clap(long, help="Only include notes created after the specified date and time")]
  created_after: Option<DateTime<Local>>,
  #[clap(long, help="Only include notes updated before the specified date and time")]
  updated_before: Option<DateTime<Local>>,
  #[clap(long, help="Only include notes updated after the specified date and time")]
  updated_after: Option<DateTime<Local>>,
}

#[derive(Args, Debug)]
pub struct FetchNoteOptions {
  #[clap(long, help="The note to fetch")]
  id: Vec<i32>
}

#[derive(Args, Debug)]
pub struct UpdateNoteOptions {
  #[clap(long, help="The note to fetch")]
  id: i32,
  #[clap(long, help="A brief summary of the note")]
  summary: Option<String>,
  #[clap(long, help="Read the note content from the specified file; use '-' for STDIN")]
  detail: Option<String>,
  #[clap(long, help="The commit SHA the note refers to")]
  commit: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteNoteOptions {
  #[clap(long, help="The note to delete")]
  id: i32,
}

pub fn note(opts: &Options, note: &NoteOptions, conn: Connection) -> Result<(), error::Error> {
  match &note.command {
    NoteCommand::Create(sub)  => create_note(opts, note, sub, conn),
    NoteCommand::List(sub)    => list_note(opts, note, sub, conn),
    NoteCommand::Fetch(sub)   => fetch_note(opts, note, sub, conn),
    NoteCommand::Update(sub)  => update_note(opts, note, sub, conn),
    NoteCommand::Delete(sub)  => delete_note(opts, note, sub, conn),
  }
}

fn create_note(opts: &Options, _note: &NoteOptions, create: &CreateNoteOptions, conn: Connection) -> Result<(), error::Error> {
  let mut val = note::Note{
    id: 0,
    creator_id: opts.agent()?.to_owned(),
    commit_sha: create.commit.to_owned(),
    summary: create.summary.to_owned(),
    detail: cli::read_input(&create.detail)?,
    data: None,
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
  };

  let mut stmt = conn.prepare("
    INSERT INTO note (
      creator_id, commit_sha, summary, detail, data, created_at, updated_at
    ) VALUES (
      ?1, ?2, ?3, ?4, ?5, ?6, ?7
    )
    RETURNING id"
  )?;

  let mut vals_iter = stmt.query_map(rusqlite::params![
    &val.creator_id, &val.commit_sha, &val.summary, &val.detail, &val.data, &val.created_at, &val.updated_at
  ], |row| {
    row.get::<usize, i32>(0)
  })?;
  let val_id = match vals_iter.next() {
    Some(next) => next?,
    None       => return Err(error::Error::ArgumentError("No identifier returned".to_owned())),
  };

  val.id = val_id;

  println!("{}", val.formatted(&opts.format()));
  Ok(())
}

fn list_note(opts: &Options, _note: &NoteOptions, list: &ListNoteOptions, conn: Connection) -> Result<(), error::Error> {
  let mut query = sqlx::Query::new();
  query
    .push("SELECT id, creator_id, commit_sha, summary, ")
    .push(if opts.verbose { "detail" } else { "NULL" })
    .push(", data, created_at, updated_at FROM note");

  if let Some(creator) = &list.creator {
    query.push_where("creator_id IN ").push_list(creator);
  }
  if let Some(commit) = &list.commit {
    query.push_where("commit_sha IN ").push_list(commit);
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

  // notes are listed chronologically
  query.push(" ORDER BY created_at");

  if opts.debug {
    eprintln!("query: {}", &query);
  }

  let mut stmt = conn.prepare(&query.sql)?;
  let vals_iter = stmt.query_map(rusqlite::params_from_iter(query.args), note::from_row(&conn))?;

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

fn fetch_note(opts: &Options, _note: &NoteOptions, fetch: &FetchNoteOptions, conn: Connection) -> Result<(), error::Error> {
  for id in &fetch.id {
    println!("{}", note::fetch(&conn, *id)?.formatted(&opts.format()));
  }
  Ok(())
}

fn update_note(opts: &Options, _note: &NoteOptions, update: &UpdateNoteOptions, conn: Connection) -> Result<(), error::Error> {
  let mut val = note::fetch(&conn, update.id)?;
  val.commit_sha = update.commit.to_owned().or(val.commit_sha);
  val.summary = update.summary.to_owned().unwrap_or(val.summary);
  val.detail = cli::read_input(&update.detail)?.or(val.detail);
  val.updated_at = chrono::Utc::now();

  let mut query = sqlx::Query::new();
  query
    .push("UPDATE note SET")
    .push("  commit_sha = ").push_var(val.commit_sha.to_owned())
    .push(", summary = ").push_var(val.summary.to_owned())
    .push(", detail = ").push_var(val.detail.to_owned())
    .push(", updated_at = ").push_var(val.updated_at)
    .push_where("id = ").push_var(update.id);

  if opts.debug {
    eprintln!("query: {}", &query);
  }

  conn
    .prepare(&query.sql)?
    .execute(rusqlite::params_from_iter(query.args))?;

  println!("{}", val.formatted(&opts.format()));
  Ok(())
}

fn delete_note(_opts: &Options, _note: &NoteOptions, del: &DeleteNoteOptions, conn: Connection) -> Result<(), error::Error> {
  let mut stmt = conn.prepare("
    DELETE note
    WHERE id = ?2",
  )?;

  let ops = stmt.execute(rusqlite::params![
    del.id,
  ])?;

  println!("deleted {} note", ops);
  Ok(())
}
