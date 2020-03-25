use std::{fmt, result, str::FromStr};

pub type Result<T> = result::Result<T, Error>;

#[derive(Default, Clone)]
pub struct Tag(String);

impl From<String> for Tag {
    fn from(s: String) -> Tag {
        Tag(s)
    }
}

impl From<Tag> for String {
    fn from(s: Tag) -> String {
        s.0
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

// data-types and report-types can be durable.
pub trait Durable<T>: Default + Clone
where
    T: ToString + FromStr,
{
    // type name must be unique across data and reports
    fn to_type(&self) -> String;
    // a unique key across all values of any type.
    fn to_key(&self) -> String;
    // serialize data-value or report-value that can be persisted.
    fn encode(&self) -> Result<T>;
    // de-serialize data-value or report-value from bytes.
    fn decode(&mut self, from: &str) -> Result<()>;
}

pub enum Error {
    Fatal(String),
    IOError(String),
    InvalidDate(String),
    InvalidJson(String),
}
