use std::hash::Hash;
use std::collections::HashSet;

use rusqlite::{Connection};

pub fn missing_one<T: Hash + Eq>(expect: T, actual: &[T]) -> Option<T> {
  if actual.iter().any(|v| v == &expect) { None } else { Some(expect) }
}

pub fn missing<'a, T: Hash + Eq>(expect: &'a [T], actual: &[T]) -> Vec<&'a T> {
  let actual_set: HashSet<&T> = actual.iter().collect();
  expect.iter().filter(|v| !actual_set.contains(v)).collect()
}

pub fn id_from_row(_: &Connection) -> impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<i32> + '_ {
  |row| {
    Ok(row.get(0)?)
  }
}
