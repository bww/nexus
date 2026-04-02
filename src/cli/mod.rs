use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Format {
  Text,
  JSON,
}

pub struct Formatted<'a, T> {
  pub value: &'a T,
  pub format: &'a Format,
}

impl Display for Format {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Text => write!(f, "text"),
      Self::JSON => write!(f, "json"),
    }
  }
}

impl FromStr for Format {
  type Err = String;
  fn from_str(input: &str) -> Result<Self, Self::Err> {
    match input {
      "text" => Ok(Format::Text),
      "json" => Ok(Format::JSON),
      _      => Err(format!("Invalid format: {}", input)),
    }
  }
}
