use std::{fmt, result, str::FromStr};

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

pub trait Store<T>
where
    T: ToString + FromStr,
{
    fn put<V>(&self, value: V) -> Result<Option<V>>
    where
        V: Durable<T>;

    fn get<V>(&self, key: &str) -> Result<V>
    where
        V: Durable<T>;

    fn delete<V>(&self, key: &str) -> Result<V>
    where
        V: Durable<T>;
}

pub enum Error {
    KeyNotFound(String),
    Fatal(String),
    IOError(String),
    InvalidDate(String),
    InvalidJson(String),
    InvalidFile(String),
    ConvertFail(String),
    NoEdit(String),
    NotFound(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match self {
            Error::KeyNotFound(msg) => write!(f, "KeyNotFound:{}", msg),
            Error::Fatal(msg) => write!(f, "Fatal:{}", msg),
            Error::IOError(msg) => write!(f, "IOError:{}", msg),
            Error::InvalidDate(msg) => write!(f, "InvalidDate:{}", msg),
            Error::InvalidJson(msg) => write!(f, "InvalidJson:{}", msg),
            Error::InvalidFile(msg) => write!(f, "InvalidFile:{}", msg),
            Error::ConvertFail(msg) => write!(f, "ConvertFail:{}", msg),
            Error::NoEdit(msg) => write!(f, "NoEdit:{}", msg),
            Error::NotFound(msg) => write!(f, "NotFound:{}", msg),
        }
    }
}
