use std::{fmt, result};

use crate::types;

pub type Result<T> = result::Result<T, Error>;

// data-types and report-types can be durable.
pub trait Durable: Default + Clone {
    // type name must be unique across data and reports
    fn to_type(&self) -> String;
    // a unique key across all values of any type.
    fn to_key(&self) -> String;
    // serialize data-value or report-value that can be persisted.
    fn encode(&self) -> Result<String>;
    // de-serialize data-value or report-value from bytes.
    fn decode(&mut self, from: &str) -> Result<()>;
}

pub trait Store: Sized {
    type Txn: Transaction<Self>;

    fn put<V>(&mut self, value: V) -> Result<Option<V>>
    where
        V: Durable;

    fn get<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable;

    fn delete<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable;

    fn iter<V>(&mut self) -> Result<Box<dyn Iterator<Item = Result<V>>>>
    where
        V: 'static + Durable;

    fn iter_journal(
        &mut self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Box<dyn Iterator<Item = Result<types::JournalEntry>>>>;

    fn commit(&mut self) -> Result<()>;

    fn pull(&mut self) -> Result<()>;

    fn push(&mut self) -> Result<()>;

    fn begin(self) -> Result<Self::Txn>;
}

pub trait Transaction<S>: Sized
where
    S: Store,
{
    fn put<V>(&mut self, value: V) -> Result<Option<V>>
    where
        V: Durable;

    fn get<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable;

    fn delete<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable;

    fn iter<V>(&mut self) -> Result<Box<dyn Iterator<Item = Result<V>>>>
    where
        V: 'static + Durable;

    fn iter_journal(
        &mut self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Box<dyn Iterator<Item = Result<types::JournalEntry>>>>;

    fn end(&mut self) -> Result<S>;
}

#[derive(Clone)]
pub enum Error {
    KeyNotFound(String),
    Fatal(String),
    IOError(String),
    InvalidDate(String),
    InvalidJson(String),
    InvalidFile(String),
    InvalidInput(String),
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
            Error::InvalidInput(msg) => write!(f, "InvalidInput:{}", msg),
            Error::ConvertFail(msg) => write!(f, "ConvertFail:{}", msg),
            Error::NoEdit(msg) => write!(f, "NoEdit:{}", msg),
            Error::NotFound(msg) => write!(f, "NotFound:{}", msg),
        }
    }
}
