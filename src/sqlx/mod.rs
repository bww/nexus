#[derive(Debug)]
pub struct StringVec(pub Vec<String>);

impl StringVec {
  pub fn from(val: &Vec<String>) -> Self {
    StringVec(val.clone())
  }

  pub fn from_option(val: &Option<Vec<String>>) -> Option<Self> {
    val.as_ref().map(Self::from)
  }
}

impl rusqlite::types::ToSql for StringVec {
  fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
    let json = serde_json::to_string(&self.0)
      .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    Ok(rusqlite::types::ToSqlOutput::Owned(
      rusqlite::types::Value::Text(json),
    ))
  }
}

impl rusqlite::types::FromSql for StringVec {
  fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
    let s = value.as_str()?;
    serde_json::from_str(s)
      .map(StringVec)
      .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
  }
}

#[macro_export]
macro_rules! sql_index {
  ($args:expr) => {
    $args.len() + 1
  };
}

#[macro_export]
macro_rules! sql_where {
  ($query:expr, $args:expr) => {
    if $args.len() == 0 {
      $query.push_str(" WHERE ");
    } else {
      $query.push_str(" AND ");
    }
  };
}

#[macro_export]
macro_rules! sql_list {
  ($query:expr, $args:expr, $list:expr) => {
    let mut n = 0;
    $query.push_str("(");
    for elem in $list {
      if n > 0 {
        $query.push_str(", ");
      }
      $query.push_str(&format!("?{}", $args.len() + 1));
      $args.push(elem);
      n = n + 1;
    }
    $query.push_str(")");
  };
}
