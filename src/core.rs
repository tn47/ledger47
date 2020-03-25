use std::{result, str::FromStr};

pub type Result<T> = result::Result<T, Error>;

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
