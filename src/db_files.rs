use chrono::{self, offset::TimeZone, Datelike};

use std::{ffi, fs, path};

use crate::{
    core::{Durable, Error, Result, Store},
    types::{self, Workspace},
};

// TODO: what is here is a crash when calling put() API ?

#[derive(Clone)]
pub struct FileLoc(ffi::OsString);

impl FileLoc {
    fn new(parent: &ffi::OsStr, file_name: &ffi::OsStr) -> FileLoc {
        let mut pp = path::PathBuf::new();
        pp.push(parent);
        pp.push(file_name);
        FileLoc(pp.into_os_string())
    }

    fn from_value<V>(parent: &ffi::OsStr, value: &V) -> FileLoc
    where
        V: Durable,
    {
        let mut pp = path::PathBuf::new();
        pp.push(parent);
        pp.push(&format!("{}.json", value.to_key()));
        FileLoc(pp.into_os_string())
    }

    fn from_key(parent: &ffi::OsStr, key: &str) -> FileLoc {
        let mut pp = path::PathBuf::new();
        pp.push(parent);
        pp.push(&format!("{}.json", key));
        FileLoc(pp.into_os_string())
    }

    fn from_journal_key(parent: &ffi::OsStr, key: &str) -> Result<FileLoc> {
        let parts: Vec<&str> = key.split("-").collect();
        if parts.len() >= 3 {
            let mut pp = path::PathBuf::new();
            pp.push(parent);
            pp.push(parts[0]);
            pp.push(parts[1]);
            pp.push(parts[2]);
            pp.push(&format!("{}.json", key));
            Ok(FileLoc(pp.into_os_string()))
        } else {
            err_at!(InvalidFile, msg: format!("journalkey:{}", key))
        }
    }

    fn create_dir_all(&self) -> Result<()> {
        match path::Path::new(&self.0).parent() {
            Some(parent) => err_at!(IOError, fs::create_dir_all(&parent))?,
            None => err_at!(InvalidFile, msg: format!("{:?}", self.0))?,
        }

        Ok(())
    }
}

impl FileLoc {
    fn put<V>(&self, value: V) -> Result<Option<V>>
    where
        V: Durable,
    {
        let old_value = self.to_value().ok();
        err_at!(IOError, fs::write(&self.0, value.encode()?.as_bytes()))?;

        Ok(old_value)
    }

    fn get<V>(&self) -> Result<V>
    where
        V: Durable,
    {
        self.to_value()
    }

    fn delete<V>(self) -> Result<V>
    where
        V: Durable,
    {
        let value = self.to_value()?;
        err_at!(IOError, fs::remove_file(&self.0))?;
        Ok(value)
    }
}

impl FileLoc {
    fn to_old_version(&self) -> FileLoc {
        let mut old = self.clone();
        old.0.push(".old");
        old
    }

    fn to_value<V>(&self) -> Result<V>
    where
        V: Durable,
    {
        let mut value: V = Default::default();
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

pub struct Db {
    dir: ffi::OsString,
    w: Workspace,
}

impl Db {
    pub fn open(dir: &ffi::OsStr) -> Result<Db> {
        let w_dir = path::Path::new(dir);
        if w_dir.exists() {
            let file_loc = FileLoc::from_key(dir, "workspace");
            let value: Workspace = file_loc.to_value()?;
            Ok(Db {
                dir: w_dir.as_os_str().to_os_string(),
                w: value,
            })
        } else {
            err_at!(NotFound, msg: format!("dir:{:?}", dir))?
        }
    }

    pub fn create(dir: ffi::OsString, value: Workspace) -> Result<Db> {
        let db = Db {
            dir: dir.clone(),
            w: value,
        };
        err_at!(IOError, fs::create_dir_all(&dir))?;
        err_at!(IOError, fs::create_dir_all(&db.to_metadata_dir().0))?;
        err_at!(IOError, fs::create_dir_all(&db.to_journal_dir().0))?;

        let file_loc = FileLoc::from_key(&dir, "workspace");
        file_loc.put(db.w.clone())?;

        Ok(db)
    }
}

impl Db {
    pub fn to_metadata_dir(&self) -> MetadataDir {
        let mut pp = path::PathBuf::new();
        pp.push(&self.dir);
        pp.push("metadata");
        MetadataDir(pp.into_os_string())
    }

    pub fn to_journal_dir(&self) -> JournalDir {
        let mut pp = path::PathBuf::new();
        pp.push(path::Path::new(&self.dir));
        pp.push("journal");
        JournalDir(pp.into_os_string())
    }
}

impl Store for Db {
    fn put<V>(&mut self, value: V) -> Result<Option<V>>
    where
        V: Durable,
    {
        match value.to_type().as_str() {
            "company" | "commodity" | "ledger" => {
                let meta_dir = self.to_metadata_dir();
                meta_dir.put(value)
            }
            "transaction" => {
                let jrn_dir = self.to_journal_dir();
                jrn_dir.put(value)
            }
            _ => err_at!(Fatal, msg: format!("unreachable"))?,
        }
    }

    fn get<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable,
    {
        let value: V = Default::default();

        match value.to_type().as_str() {
            "company" | "commodity" | "ledger" => {
                let meta_dir = self.to_metadata_dir();
                meta_dir.get(key)
            }
            "transaction" => {
                let jrn_dir = self.to_journal_dir();
                jrn_dir.get(key)
            }
            _ => err_at!(Fatal, msg: format!("unreachable"))?,
        }
    }

    fn delete<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable,
    {
        let value: V = Default::default();

        match value.to_type().as_str() {
            "company" | "commodity" | "ledger" => {
                let meta_dir = self.to_metadata_dir();
                meta_dir.delete(key)
            }
            "transaction" => {
                let jrn_dir = self.to_journal_dir();
                jrn_dir.delete(key)
            }
            _ => err_at!(Fatal, msg: format!("unreachable"))?,
        }
    }

    fn iter<V>(&mut self) -> Result<Box<dyn Iterator<Item = Result<V>>>>
    where
        V: 'static + Durable,
    {
        let iter = self.to_metadata_dir().iter()?;
        Ok(Box::new(iter))
    }

    fn iter_transaction(
        &mut self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Box<dyn Iterator<Item = Result<types::Transaction>>>> {
        let iter = self.to_journal_dir().iter(from, to)?;
        Ok(Box::new(iter))
    }
}

pub struct MetadataDir(ffi::OsString);

impl MetadataDir {
    const TYPES: [&'static str; 3] = ["company", "commodity", "ledger"];

    pub fn put<V>(&self, value: V) -> Result<Option<V>>
    where
        V: Durable,
    {
        let typ = value.to_type();
        if !Self::TYPES.contains(&typ.as_str()) {
            err_at!(Fatal, msg: format!("invalid type:{}", typ))?;
        }

        let file_loc = FileLoc::from_value(&self.0, &value);
        file_loc.put(value)
    }

    pub fn get<V>(&self, key: &str) -> Result<V>
    where
        V: Durable,
    {
        let value: V = Default::default();

        let typ = value.to_type();
        if !Self::TYPES.contains(&typ.as_str()) {
            err_at!(Fatal, msg: format!("invalid type:{}", typ))?;
        }

        let file_loc = FileLoc::from_key(&self.0, key);
        file_loc.get()
    }

    pub fn delete<V>(&self, key: &str) -> Result<V>
    where
        V: Durable,
    {
        let value: V = Default::default();

        let typ = value.to_type();
        if !Self::TYPES.contains(&typ.as_str()) {
            err_at!(Fatal, msg: format!("invalid type:{}", typ))?;
        }

        let file_loc = FileLoc::from_key(&self.0, key);
        file_loc.delete()
    }

    pub fn iter<V>(&self) -> Result<std::vec::IntoIter<Result<V>>>
    where
        V: Durable,
    {
        let mut dfs = vec![];
        for item in err_at!(IOError, fs::read_dir(&self.0), format!("{:?}", self.0))? {
            let item = err_at!(IOError, item, format!("{:?}", self.0))?;
            dfs.push(Ok(FileLoc::new(&self.0, &item.file_name()).to_value()?));
        }

        Ok(dfs.into_iter())
    }
}

pub struct JournalDir(ffi::OsString);

impl JournalDir {
    const TYPES: [&'static str; 3] = ["company", "commodity", "ledger"];

    fn put<V>(&self, value: V) -> Result<Option<V>>
    where
        V: Durable,
    {
        let typ = value.to_type();
        if !Self::TYPES.contains(&typ.as_str()) {
            err_at!(Fatal, msg: format!("invalid type:{}", typ))?;
        }

        let key = value.to_key();
        let file_loc = FileLoc::from_journal_key(&self.0, &key)?;
        file_loc.create_dir_all()?;

        file_loc.put(value)
    }

    pub fn get<V>(&self, key: &str) -> Result<V>
    where
        V: Durable,
    {
        let value: V = Default::default();

        let typ = value.to_type();
        if !Self::TYPES.contains(&typ.as_str()) {
            err_at!(Fatal, msg: format!("invalid type:{}", typ))?;
        }

        let file_loc = FileLoc::from_key(&self.0, key);
        file_loc.get()
    }

    pub fn delete<V>(&self, key: &str) -> Result<V>
    where
        V: Durable,
    {
        let value: V = Default::default();

        let typ = value.to_type();
        if !Self::TYPES.contains(&typ.as_str()) {
            err_at!(Fatal, msg: format!("invalid type:{}", typ))?;
        }

        let file_loc = FileLoc::from_key(&self.0, key);
        file_loc.delete()
    }

    fn iter(
        &self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<IterTransaction> {
        Ok(IterTransaction::new(self.0.clone(), from, to))
    }
}

struct IterTransaction {
    from: chrono::DateTime<chrono::Utc>,
    to: chrono::DateTime<chrono::Utc>,
    iter: JournalYears<types::Transaction>,
    done: bool,
}

impl IterTransaction {
    fn new(
        dir: ffi::OsString,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> IterTransaction {
        let iter = JournalYears::new(dir, from.date());
        IterTransaction {
            from: from,
            to: to,
            iter,
            done: false,
        }
    }
}

impl Iterator for IterTransaction {
    type Item = Result<types::Transaction>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            loop {
                match self.iter.next() {
                    Some(res) => match res {
                        Ok(txn) if txn.created >= self.from && txn.created <= self.to => {
                            break Some(Ok(txn))
                        }
                        Ok(_) => continue,
                        Err(err) => {
                            self.done = true;
                            break Some(Err(err));
                        }
                    },
                    None => {
                        self.done = true;
                        break None;
                    }
                }
            }
        } else {
            None
        }
    }
}

struct JournalYears<V>
where
    V: Ord + Durable,
{
    journal_dir: ffi::OsString,
    from: chrono::Date<chrono::Utc>,
    years: Vec<i32>,
    year: Option<JournalYear<V>>,
}

impl<V> JournalYears<V>
where
    V: Ord + Durable,
{
    fn new(journal_dir: ffi::OsString, from: chrono::Date<chrono::Utc>) -> JournalYears<V> {
        JournalYears {
            journal_dir,
            from,
            years: (from.year()..=chrono::Utc::today().year()).collect(),
            year: Default::default(),
        }
    }
}

impl<V> Iterator for JournalYears<V>
where
    V: Ord + Durable,
{
    type Item = Result<V>;

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

struct JournalYear<V>
where
    V: Ord + Durable,
{
    year_dir: ffi::OsString,
    months: Vec<u32>,
    month: Option<JournalMonth<V>>,
}

impl<V> JournalYear<V>
where
    V: Ord + Durable,
{
    fn new(journal_dir: ffi::OsString, from: chrono::Date<chrono::Utc>) -> JournalYear<V> {
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

impl<V> Iterator for JournalYear<V>
where
    V: Ord + Durable,
{
    type Item = Result<V>;

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

struct JournalMonth<V>
where
    V: Ord + Durable,
{
    month_dir: ffi::OsString,
    days: Vec<u32>,
    day: Option<JournalDay<V>>,
}

impl<V> JournalMonth<V>
where
    V: Ord + Durable,
{
    fn new(year_dir: ffi::OsString, month: u32) -> JournalMonth<V> {
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

impl<V> Iterator for JournalMonth<V>
where
    V: Ord + Durable,
{
    type Item = Result<V>;

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

struct JournalDay<V>
where
    V: Ord + Durable,
{
    _day_dir: ffi::OsString,
    txns: Vec<V>,
}

impl<V> JournalDay<V>
where
    V: Ord + Durable,
{
    fn new(month_dir: ffi::OsString, day: u32) -> Option<JournalDay<V>> {
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

    fn new_txn(day_dir: &ffi::OsStr, item: fs::DirEntry) -> Option<V> {
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
                let mut value: V = Default::default();
                value.decode(from).ok()?;
                Some(value)
            }
            None => None,
        }
    }
}

impl<V> Iterator for JournalDay<V>
where
    V: Ord + Durable,
{
    type Item = Result<V>;

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
