use chrono::{self, Datelike};

use std::str::FromStr;

use crate::core::{Error, Result};

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

#[macro_export]
macro_rules! native_to_json_string_array {
    ($val:expr) => {
        $val.into_iter()
            .map(|s| Json::String(s.to_string()))
            .collect()
    };
}

#[macro_export]
macro_rules! json_to_native_string {
    ($j:expr, $key:expr, $msg:expr) => {
        match err_at!(InvalidJson, $j.get($key), $msg)?.as_str() {
            Some(val) => Ok(val.to_string()),
            None => err_at!(InvalidJson, msg: $msg),
        }
    };
}

#[macro_export]
macro_rules! json_to_native_string_array {
    ($j:expr, $key:expr, $msg:expr) => {
        match err_at!(InvalidJson, $j.get($key), $msg)?.to_array() {
            Some(val) => {
                let mut arr = vec![];
                for j in val.into_iter() {
                    match j.as_str() {
                        Some(s) => arr.push(s.to_string()),
                        None => err_at!(InvalidJson, msg: $msg)?,
                    }
                }
                Ok(arr)
            }
            None => err_at!(InvalidJson, msg: $msg),
        }
    };
}

pub fn date_to_period<T>(date: chrono::Date<T>) -> (chrono::Date<T>, chrono::Date<T>)
where
    T: chrono::TimeZone,
{
    let tz = date.timezone();
    let closing = tz.ymd(date.year(), 3, 31);
    if date <= closing {
        (tz.ymd(date.year() - 1, 4, 1), tz.ymd(date.year(), 3, 31))
    } else {
        (tz.ymd(date.year(), 4, 1), tz.ymd(date.year() + 1, 3, 31))
    }
}

pub fn csv<T>(s: String) -> Result<Vec<T>>
where
    T: FromStr,
{
    let mut ys = vec![];
    let xs: Vec<&str> = s.split(',').collect();
    for x in xs.into_iter() {
        match x.parse() {
            Ok(y) => Ok(ys.push(y)),
            Err(_) => Err(Error::ConvertFail("invalid csv".to_string())),
        }?
    }

    Ok(ys)
}

pub fn str_as_anuh(s: &str) -> bool {
    for ch in s.chars() {
        match ch {
            '-' | '_' => (),
            ch if ch.is_alphanumeric() => (),
            _ => return false,
        }
    }
    true
}

pub fn str_as_anuhdc(s: &str) -> bool {
    for ch in s.chars() {
        match ch {
            '-' | '_' | '.' | ':' => (),
            ch if ch.is_alphanumeric() => (),
            _ => return false,
        }
    }
    true
}
