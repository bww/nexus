use core::num;
use std::io;
use std::fmt;
use std::str;
use std::string;

use rusqlite;

#[derive(Debug, PartialEq)]
pub enum Error {
  IOError(String),
  Utf8Error(str::Utf8Error),
  FromUtf8Error(string::FromUtf8Error),
  ParseIntError(num::ParseIntError),
  RusqliteError(rusqlite::Error),
}

impl From<str::Utf8Error> for Error {
  fn from(err: str::Utf8Error) -> Self {
    Self::Utf8Error(err)
  }
}

impl From<string::FromUtf8Error> for Error {
  fn from(err: string::FromUtf8Error) -> Self {
    Self::FromUtf8Error(err)
  }
}

impl From<num::ParseIntError> for Error {
  fn from(err: num::ParseIntError) -> Self {
    Self::ParseIntError(err)
  }
}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Self {
    Self::IOError(err.to_string())
  }
}

impl From<rusqlite::Error> for Error {
  fn from(err: rusqlite::Error) -> Self {
    Self::RusqliteError(err)
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::IOError(err) => err.fmt(f),
      Self::Utf8Error(err) => err.fmt(f),
      Self::FromUtf8Error(err) => err.fmt(f),
      Self::ParseIntError(err) => err.fmt(f),
      Self::RusqliteError(err) => err.fmt(f),
    }
  }
}
