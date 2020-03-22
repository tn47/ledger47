use std::result;

pub type Result<T> = result::Result<T, Error>;

// data-types and report-types can be durable.
pub trait Durable: Default + Clone {
    // type must be unique across data and reports
    fn to_type(&self) -> String;
    // name must be unique for all values of a given data-type or
    // report-type.
    fn to_unique_name(&self) -> String;
    // serialize data-value or report-value that can be store.
    fn encode(&self, buffer: &mut Vec<u8>) -> Result<usize>;
    // de-serialize data-value or report-value from bytes.
    fn decode(&mut self, buffer: &[u8]) -> Result<usize>;
}

pub struct Tag(String);

pub enum Error {
    Fatal(String),
    IOError(String),
    InvalidDate(String),
}

#[macro_export]
macro_rules! err_at {
    ($v:ident, msg:$msg:expr) => {
        //
        Err(Error::$v(format!("{}:{} {}", file!(), line!(), $msg)))
    };
    ($v:ident, $e:expr) => {
        match $e {
            Ok(val) => Ok(val),
            Err(err) => {
                let msg = format!("{}:{} err:{}", file!(), line!(), err);
                Err(Error::$v(msg))
            }
        }
    };
    ($v:ident, $e:expr, $msg:expr) => {
        match $e {
            Ok(val) => Ok(val),
            Err(err) => {
                let msg = format!("{}:{} {} err:{}", file!(), line!(), $msg, err);
                Err(Error::$v(msg))
            }
        }
    };
}
