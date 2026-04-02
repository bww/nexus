
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
