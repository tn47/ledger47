use chrono::{self, offset::TimeZone, Datelike};
use jsondata::Json;

use std::{ffi, fs, path};

use crate::core::{Durable, Error, Result};

#[derive(Clone)]
pub struct FileLoc(ffi::OsString);

impl FileLoc {
    fn new(parent: &ffi::OsStr, file_name: &ffi::OsStr) -> FileLoc {
        let mut pp = path::PathBuf::new();
        pp.push(parent);
        pp.push(file_name);
        FileLoc(pp.into_os_string())
    }

    fn from_value<T>(value: &T, parent: &ffi::OsStr) -> FileLoc
    where
        T: Durable<Json>,
    {
        let mut pp = path::PathBuf::new();
        pp.push(parent);
        pp.push(&format!("{}.json", value.to_key()));
        FileLoc(pp.into_os_string())
    }

    fn to_old_version(&self) -> FileLoc {
        let mut old = self.clone();
        old.0.push(".old");
        old
    }

    fn to_value<T>(&self) -> Result<T>
    where
        T: Durable<Json>,
    {
        let mut value: T = Default::default();
        let typ = value.to_type();
        let data = err_at!(IOError, fs::read(&self.0), typ)?;
        let s = err_at!(InvalidJson, std::str::from_utf8(&data), typ)?;
        value.decode(s)?;
        Ok(value)
    }

    fn is_old_version(&self) -> bool {
        match self.0.to_str() {
            Some(file_loc) => file_loc.ends_with(".old"),
            None => false,
        }
    }
}

impl From<FileLoc> for ffi::OsString {
    fn from(file_loc: FileLoc) -> ffi::OsString {
        file_loc.0
    }
}

pub struct Workspace(ffi::OsString);

impl Workspace {
    pub fn to_metadata_dir(&self) -> MetadataDir {
        let mut pp = path::PathBuf::new();
        pp.push(&self.0);
        pp.push("metadata");
        MetadataDir(pp.into_os_string())
    }

    //pub fn to_journal_dir(&self) -> JournalDir {
    //    let mut pp = path::PathBuf::new();
    //    pp.push(path::Path::new(&self.0));
    //    pp.push("journal");
    //    MetadataDir(pp.into_os_string())
    //}

    //pub fn to_report_dir(&self) -> ReportDir {
    //    let mut pp = path::PathBuf::new();
    //    pp.push(path::Path::new(&self.0));
    //    pp.push("report");
    //    MetadataDir(pp.into_os_string())
    //}
}

pub struct MetadataDir(ffi::OsString);

impl MetadataDir {
    const TYPES: [&'static str; 3] = ["company", "commodity", "ledger"];

    pub fn new_file<T>(&self, value: T) -> Result<DataFile<T>>
    where
        T: Durable<Json>,
    {
        if !Self::TYPES.contains(&value.to_type().as_str()) {
            err_at!(Fatal, msg: format!("invalid type:{}", value.to_type()))?;
        }
        DataFile::new(FileLoc::from_value(&value, &self.0), value)
    }

    pub fn iter<T>(&self) -> Result<impl Iterator<Item = DataFile<T>>>
    where
        T: Durable<Json>,
    {
        let mut dfs = vec![];
        for item in err_at!(IOError, fs::read_dir(&self.0), format!("{:?}", self.0))? {
            let item = err_at!(IOError, item, format!("{:?}", self.0))?;

            let file_loc = FileLoc::new(&self.0, &item.file_name());
            let value: Result<T> = file_loc.to_value();
            match value.ok() {
                Some(value) => dfs.push(DataFile::new(file_loc, value)?),
                None => (),
            }
        }

        Ok(dfs.into_iter())
    }
}

#[derive(Clone)]
pub enum DataFile<T: Durable<Json>> {
    Company { file_loc: FileLoc, value: T },
    Ledger { file_loc: FileLoc, value: T },
    Commodity { file_loc: FileLoc, value: T },
    Transaction { file_loc: FileLoc, value: T },
}

impl<T: Durable<Json>> DataFile<T> {
    pub fn new(file_loc: FileLoc, value: T) -> Result<DataFile<T>> {
        match value.to_type().as_str() {
            "company" => Ok(DataFile::Company { file_loc, value }),
            "ledger" => Ok(DataFile::Ledger { file_loc, value }),
            "commodity" => Ok(DataFile::Commodity { file_loc, value }),
            "transaction" => Ok(DataFile::Transaction { file_loc, value }),
            _ => err_at!(Fatal, msg: format!("unreachable")),
        }
    }

    fn to_url(&self) -> ffi::OsString {
        match self {
            DataFile::Company { file_loc, .. } => file_loc.0.clone(),
            DataFile::Ledger { file_loc, .. } => file_loc.0.clone(),
            DataFile::Commodity { file_loc, .. } => file_loc.0.clone(),
            DataFile::Transaction { file_loc, .. } => file_loc.0.clone(),
        }
    }

    pub fn to_value(&self) -> T {
        match self {
            DataFile::Company { value, .. } => value,
            DataFile::Ledger { value, .. } => value,
            DataFile::Commodity { value, .. } => value,
            DataFile::Transaction { value, .. } => value,
        }
        .clone()
    }

    fn is_old_version(&self) -> bool {
        match self {
            DataFile::Company { file_loc, .. } => file_loc,
            DataFile::Ledger { file_loc, .. } => file_loc,
            DataFile::Commodity { file_loc, .. } => file_loc,
            DataFile::Transaction { file_loc, .. } => file_loc,
        }
        .is_old_version()
    }

    fn older_version(&self, other: &DataFile<T>) -> Result<Option<ffi::OsString>> {
        if self.is_old_version() == false && other.is_old_version() == false {
            Ok(None)
        } else {
            let (file_loc, other_file_loc) = (self.to_url(), other.to_url());
            let file_loc = file_loc.to_str().unwrap().to_string();
            let other_file_loc = other_file_loc.to_str().unwrap().to_string();

            let mut file_loc1 = file_loc.clone();
            file_loc1.push_str(".old");

            let mut other_file_loc1 = file_loc.clone();
            other_file_loc1.push_str(".old");

            if file_loc1 == other_file_loc {
                Ok(Some(other_file_loc.into()))
            } else if other_file_loc1 == file_loc {
                Ok(Some(file_loc.into()))
            } else {
                Ok(None)
            }
        }
    }

    fn swap_value(&mut self, value: T) -> T {
        let v_ref = match self {
            DataFile::Company { value, .. } => value,
            DataFile::Commodity { value, .. } => value,
            DataFile::Ledger { value, .. } => value,
            DataFile::Transaction { value, .. } => value,
        };
        let old_value = v_ref.clone();
        *v_ref = value;
        old_value
    }

    pub fn put(&mut self, value: T) -> Result<T> {
        let js_value = value.encode()?;

        let old_value = self.swap_value(value);
        let old_file_loc = {
            let mut old_file_loc = self.to_url().clone();
            old_file_loc.push(".old");
            old_file_loc
        };

        err_at!(IOError, fs::rename(&self.to_url(), &old_file_loc))?;
        err_at!(
            IOError,
            fs::write(&self.to_url(), js_value.to_string().as_bytes())
        )?;
        err_at!(IOError, fs::remove_file(&old_file_loc))?;

        Ok(old_value)
    }

    pub fn delete(self) -> Result<()> {
        let file_loc: ffi::OsString = match self {
            DataFile::Company { file_loc, .. } => file_loc,
            DataFile::Ledger { file_loc, .. } => file_loc,
            DataFile::Commodity { file_loc, .. } => file_loc,
            DataFile::Transaction { file_loc, .. } => file_loc,
        }
        .into();
        err_at!(IOError, fs::remove_file(file_loc))
    }
}

pub struct JournalYears<T>
where
    T: Ord + Durable<Json>,
{
    journal_dir: ffi::OsString,
    from: chrono::Date<chrono::Utc>,
    years: Vec<i32>,
    year: Option<JournalYear<T>>,
}

impl<T> JournalYears<T>
where
    T: Ord + Durable<Json>,
{
    pub fn new(journal_dir: ffi::OsString, from: chrono::Date<chrono::Utc>) -> JournalYears<T> {
        JournalYears {
            journal_dir,
            from,
            years: (from.year()..=chrono::Utc::today().year()).collect(),
            year: Default::default(),
        }
    }
}

impl<T> Iterator for JournalYears<T>
where
    T: Ord + Durable<Json>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.year.take() {
                Some(mut year) => match year.next() {
                    Some(item) => {
                        self.year = Some(year);
                        break Some(item);
                    }
                    None => (),
                },
                None if self.years.len() == 0 => break None,
                None => {
                    let from = self.from.with_year(self.years.remove(0)).unwrap();
                    self.year = Some(JournalYear::new(self.journal_dir.clone(), from));
                }
            }
        }
    }
}

pub struct JournalYear<T>
where
    T: Ord + Durable<Json>,
{
    year_dir: ffi::OsString,
    months: Vec<u32>,
    month: Option<JournalMonth<T>>,
}

impl<T> JournalYear<T>
where
    T: Ord + Durable<Json>,
{
    pub fn new(journal_dir: ffi::OsString, from: chrono::Date<chrono::Utc>) -> JournalYear<T> {
        let year_dir = {
            let mut pp = path::PathBuf::new();
            pp.push(path::Path::new(&journal_dir));
            pp.push(&from.year().to_string());
            pp.into_os_string()
        };

        JournalYear {
            year_dir,
            months: (from.month()..=chrono::Utc::today().month()).collect(),
            month: Default::default(),
        }
    }
}

impl<T> Iterator for JournalYear<T>
where
    T: Ord + Durable<Json>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.month.take() {
                Some(mut month) => match month.next() {
                    Some(item) => {
                        self.month = Some(month);
                        break Some(item);
                    }
                    None => (),
                },
                None if self.months.len() == 0 => break None,
                None => {
                    let month = self.months.remove(0);
                    self.month = Some(JournalMonth::new(self.year_dir.clone(), month));
                }
            }
        }
    }
}

pub struct JournalMonth<T>
where
    T: Ord + Durable<Json>,
{
    month_dir: ffi::OsString,
    days: Vec<u32>,
    day: Option<JournalDay<T>>,
}

impl<T> JournalMonth<T>
where
    T: Ord + Durable<Json>,
{
    pub fn new(year_dir: ffi::OsString, month: u32) -> JournalMonth<T> {
        let month_dir = {
            let mut pp = path::PathBuf::new();
            pp.push(path::Path::new(&year_dir));
            pp.push(&month.to_string());
            pp.into_os_string()
        };
        JournalMonth {
            month_dir,
            days: (1..32).collect(),
            day: Default::default(),
        }
    }
}

impl<T> Iterator for JournalMonth<T>
where
    T: Ord + Durable<Json>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.day.take() {
                Some(mut day) => match day.next() {
                    Some(item) => {
                        self.day = Some(day);
                        break Some(item);
                    }
                    None => (),
                },
                None if self.days.len() == 0 => break None,
                None => {
                    let day = self.days.remove(0);
                    match JournalDay::new(self.month_dir.clone(), day) {
                        Some(day) => self.day = Some(day),
                        None => (),
                    }
                }
            }
        }
    }
}

pub struct JournalDay<T>
where
    T: Ord + Durable<Json>,
{
    _day_dir: ffi::OsString,
    txns: Vec<T>,
}

impl<T> JournalDay<T>
where
    T: Ord + Durable<Json>,
{
    pub fn new(month_dir: ffi::OsString, day: u32) -> Option<JournalDay<T>> {
        let day_dir = {
            let mut pp = path::PathBuf::new();
            pp.push(path::Path::new(&month_dir));
            pp.push(&day.to_string());
            pp.into_os_string()
        };
        let mut txns = vec![];
        for item in err_at!(IOError, fs::read_dir(&day_dir)).ok()? {
            let item = err_at!(IOError, item).ok()?;
            match Self::new_txn(&day_dir, item) {
                Some(txn) => txns.push(txn),
                None => continue,
            }
        }
        txns.sort();

        Some(JournalDay {
            _day_dir: day_dir,
            txns,
        })
    }

    fn new_txn(day_dir: &ffi::OsStr, item: fs::DirEntry) -> Option<T> {
        match item.file_name().to_str() {
            Some(file_name) => {
                let file_loc = {
                    let mut pp = path::PathBuf::new();
                    pp.push(path::Path::new(day_dir));
                    pp.push(&file_name);
                    pp.into_os_string()
                };
                let data = fs::read(&file_loc).ok()?;
                let from = std::str::from_utf8(&data).ok()?;
                let mut value: T = Default::default();
                value.decode(from).ok()?;
                Some(value)
            }
            None => None,
        }
    }
}

impl<T> Iterator for JournalDay<T>
where
    T: Ord + Durable<Json>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.txns.len() {
            0 => None,
            _ => Some(Ok(self.txns.remove(0))),
        }
    }
}

fn days_in_month(year: i32, month: u32) -> Vec<chrono::Date<chrono::Utc>> {
    let mut start_date = chrono::Utc.ymd(year, month, 1);
    let mut dates = vec![];
    loop {
        dates.push(start_date);
        match start_date.succ_opt() {
            Some(next_date) => {
                start_date = next_date;
            }
            None => break dates,
        }
    }
}
