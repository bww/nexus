use clap::{Subcommand, Args};
use rusqlite::{Connection, Result};

use crate::error;
use crate::cli;
use crate::{sql_where, sql_list};
use crate::note;
use crate::Options;

#[derive(Args, Debug)]
pub struct NoteOptions {
  #[clap(long, env="NEXUS_AGENT", help="A unique identifier of the agent operating on the project (use: 'agent new' to assign a new identifier)")]
  agent: String,
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
}

#[derive(Args, Debug)]
pub struct FetchNoteOptions {
  #[clap(long, help="The note to fetch")]
  id: i32
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

fn note_row(_conn: &Connection) -> impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<note::Note> + '_ {
  |row| {
    Ok(note::Note {
      id: row.get(0)?,
      creator_id: row.get(1)?,
      commit_sha: row.get(2)?,
      summary: row.get(3)?,
      detail: row.get(4)?,
      data: row.get(5)?,
      created_at: row.get(6)?,
      updated_at: row.get(7)?,
    })
  }
}

fn create_note(opts: &Options, note: &NoteOptions, create: &CreateNoteOptions, conn: Connection) -> Result<(), error::Error> {
  let mut val = note::Note{
    id: 0,
    creator_id: note.agent.to_owned(),
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
  let mut query = "
    SELECT id, creator_id, commit_sha, summary, detail, data, created_at, updated_at
    FROM note".to_string();

  let mut args: Vec<&dyn rusqlite::types::ToSql> = vec![];

  if let Some(creator) = &list.creator {
    sql_where!(query, args);
    query.push_str("creator_id IN ");
    sql_list!(query, args, creator);
  }

  if let Some(commit) = &list.commit {
    sql_where!(query, args);
    query.push_str("commit_sha IN ");
    sql_list!(query, args, commit);
  }

  query.push_str(" ORDER BY updated_at DESC");

  let mut stmt = conn.prepare(&query)?;
  let vals_iter = stmt.query_map(rusqlite::params_from_iter(args.iter()), note_row(&conn))?;

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
  let mut stmt = conn.prepare("
    SELECT id, creator_id, commit_sha, summary, detail, data, created_at, updated_at
    FROM note
    WHERE id = ?1")?;

  let val = stmt.query_one(rusqlite::params![fetch.id], note_row(&conn))?;

  println!("{}", val.formatted(&opts.format()));
  Ok(())
}

fn update_note(opts: &Options, _note: &NoteOptions, update: &UpdateNoteOptions, conn: Connection) -> Result<(), error::Error> {
  let mut stmt = conn.prepare("
    SELECT id, creator_id, commit_sha, summary, detail, data, created_at, updated_at
    FROM note
    WHERE id = ?1")?;

  let mut val = stmt.query_one(rusqlite::params![update.id], note_row(&conn))?;
  val.commit_sha = update.commit.to_owned().or(val.commit_sha);
  val.summary = update.summary.to_owned().unwrap_or(val.summary);
  val.detail = cli::read_input(&update.detail)?.or(val.detail);
  val.updated_at = chrono::Utc::now();

  let mut stmt = conn.prepare("
    UPDATE note SET commit_sha = ?1, summary = ?2, detail = ?3, updated_at = ?4
    WHERE id = ?5"
  )?;
  stmt.execute(rusqlite::params![
    &val.commit_sha, &val.summary, &val.detail, &val.updated_at,
    update.id,
  ])?;

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
