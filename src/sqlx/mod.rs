use std::fmt;

use rusqlite;

pub mod results;

pub struct Query {
  pub sql: String,
  pub args: Vec<Box<dyn rusqlite::types::ToSql>>,
  n_where: usize,
}

impl Query {
  pub fn new() -> Self {
    Query{
      sql: String::new(),
      args: Vec::new(),
      n_where: 0,
    }
  }

  pub fn push(&mut self, query: &str) -> &mut Self {
    self.sql.push_str(query);
    self
  }

  pub fn push_where(&mut self, tail: &str) -> &mut Self {
    if self.n_where == 0 {
      self.sql.push_str(" WHERE ");
    } else {
      self.sql.push_str(" AND ");
    }
    self.n_where += 1;
    self.sql.push_str(tail);
    self
  }

  pub fn push_var<T: rusqlite::types::ToSql + 'static>(&mut self, arg: T) -> &mut Self {
    self.sql.push_str(&format!("?{}", self.args.len() + 1));
    self.args.push(Box::new(arg));
    self
  }

  pub fn push_list<T: rusqlite::types::ToSql + Clone + 'static>(&mut self, list: &[T]) -> &mut Self {
    self.sql.push_str("(");
    for (i, elem) in list.iter().enumerate() {
      if i > 0 {
        self.sql.push_str(", ");
      }
      self.sql.push_str(&format!("?{}", self.args.len() + 1));
      self.args.push(Box::new(elem.clone()));
    }
    self.sql.push_str(")");
    self
  }
}

impl fmt::Display for Query {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", &self.sql)
  }
}
