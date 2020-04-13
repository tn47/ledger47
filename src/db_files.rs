use chrono::{self, offset::TimeZone, Datelike};
use git2;
use log::trace;

use std::{ffi, fs, path};

use crate::{
    core::{Durable, Error, Result, Store, Transaction},
    types,
};

// TODO: add git description.

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
}

impl From<FileLoc> for ffi::OsString {
    fn from(file_loc: FileLoc) -> ffi::OsString {
        file_loc.0
    }
}

pub struct Db {
    dir: ffi::OsString,
    w: types::Workspace,
    repo: Option<git2::Repository>,
    remotes: Vec<git2::Repository>,
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

    fn get_head_commit(&self) -> Result<git2::Commit> {
        let old_head_oid = err_at!(
            IOError,
            self.repo.as_ref().unwrap().refname_to_id("HEAD"),
            format!("git refname_to_id")
        )?;
        let parent = err_at!(
            IOError,
            self.repo.as_ref().unwrap().find_commit(old_head_oid),
            format!("git find_commit")
        )?;

        Ok(parent)
    }

    fn do_commit(&mut self, message: &str) -> Result<(git2::Oid, git2::Oid)> {
        // stage the changes.
        let mut index = err_at!(
            IOError,
            self.repo.as_ref().unwrap().index(),
            format!("git error")
        )?;
        err_at!(
            IOError,
            index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None),
            format!("git add_all")
        )?;
        let oid = err_at!(IOError, index.write_tree(), format!("git write"))?;

        // commit the staged changs.
        let tree = err_at!(
            IOError,
            self.repo.as_ref().unwrap().find_tree(oid),
            format!("git find_tree")
        )?;
        let old_head_oid = err_at!(
            IOError,
            self.repo.as_ref().unwrap().refname_to_id("HEAD"),
            format!("git refname_to_id")
        )?;
        let parent = err_at!(
            IOError,
            self.repo.as_ref().unwrap().find_commit(old_head_oid),
            format!("git find_commit")
        )?;
        let signature = err_at!(
            IOError,
            self.repo.as_ref().unwrap().signature(),
            format!("git signature")
        )?;
        let new_head_oid = err_at!(
            IOError,
            self.repo.as_ref().unwrap().commit(
                Some("HEAD"), /*update_ref*/
                &signature,   /*author*/
                &signature,   /*committer*/
                message,
                &tree,
                &[&parent],
            ),
            format!("git commit")
        )?;

        trace!("git commit {}->{}", old_head_oid, new_head_oid);

        Ok((old_head_oid, new_head_oid))
    }
}

impl Store for Db {
    type Txn = DbTransaction;

    fn create(dir: &ffi::OsStr, w: types::Workspace) -> Result<Db> {
        let repo = err_at!(
            IOError,
            git2::Repository::init(dir),
            format!("can't initialise git repository: {:?}", dir)
        )?;

        let mut remotes = vec![];
        for remote in w.remotes.iter() {
            let remote: &ffi::OsStr = remote.as_ref();
            remotes.push(err_at!(
                IOError,
                git2::Repository::open(remote),
                format!("can't open remote git repository: {:?}", remote)
            )?);
        }

        let mut db = Db {
            dir: dir.to_os_string(),
            w,
            repo: Some(repo),
            remotes,
        };
        err_at!(IOError, fs::create_dir_all(&dir))?;
        err_at!(IOError, fs::create_dir_all(&db.to_metadata_dir().0))?;
        err_at!(IOError, fs::create_dir_all(&db.to_journal_dir().0))?;

        let file_loc = FileLoc::from_key(&dir, "workspace");
        file_loc.put(db.w.clone())?;

        db.do_commit("user commit")?;

        Ok(db)
    }

    fn open(dir: &ffi::OsStr) -> Result<Db> {
        let w_dir = path::Path::new(dir);
        if w_dir.exists() {
            let file_loc = FileLoc::from_key(dir, "workspace");
            let w: types::Workspace = file_loc.to_value()?;

            let repo = err_at!(
                IOError,
                git2::Repository::open(dir),
                format!("can't open git repository: {:?}", dir)
            )?;

            let mut remotes = vec![];
            for remote in w.remotes.iter() {
                let remote: &ffi::OsStr = remote.as_ref();
                remotes.push(err_at!(
                    IOError,
                    git2::Repository::open(remote),
                    format!("can't open remote git repository: {:?}", remote)
                )?);
            }

            let mut db = Db {
                dir: w_dir.as_os_str().to_os_string(),
                w,
                repo: Some(repo),
                remotes,
            };

            // check for broken transactions.
            {
                let head_commit = db.get_head_commit()?;
                if head_commit.message().unwrap().starts_with("txn commit") {
                    let parent = err_at!(
                        //
                        IOError, head_commit.parent(0), format!("git parent")
                    )?;
                    let mut cob = git2::build::CheckoutBuilder::new();
                    cob.force();
                    err_at!(
                        IOError,
                        db.repo.as_ref().unwrap().reset(
                            parent.as_object(),
                            git2::ResetType::Hard,
                            Some(&mut cob)
                        ),
                        format!("git reset")
                    )?;
                }
            }

            db.w.set_txn_uuid(0);
            db.put(db.w.clone())?;
            db.do_commit("user commit")?;

            Ok(db)
        } else {
            err_at!(NotFound, msg: format!("dir:{:?}", dir))?
        }
    }

    fn put<V>(&mut self, value: V) -> Result<Option<V>>
    where
        V: Durable,
    {
        match value.to_type().as_str() {
            "company" | "commodity" | "ledger" => {
                let meta_dir = self.to_metadata_dir();
                meta_dir.put(value)
            }
            "journalentry" => {
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
            "journalentry" => {
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
            "journalentry" => {
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

    fn iter_journal(
        &mut self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Box<dyn Iterator<Item = Result<types::JournalEntry>>>> {
        let iter = self.to_journal_dir().iter(from, to)?;
        Ok(Box::new(iter))
    }

    fn commit(&mut self) -> Result<()> {
        self.do_commit("user commit")?;
        Ok(())
    }

    fn pull(&mut self) -> Result<()> {
        todo!()
    }

    fn push(&mut self) -> Result<()> {
        todo!()
    }

    fn begin(mut self) -> Result<DbTransaction> {
        let uuid = uuid::Uuid::new_v4().as_u128();
        self.w.set_txn_uuid(uuid);
        self.put(self.w.clone())?;

        let (oh_oid, nh_oid) = self.do_commit(&format!("txn commit {}", uuid))?;
        trace!("git txn-commit {}->{}", oh_oid, nh_oid);

        Ok(DbTransaction {
            uuid,
            old_head_oid: oh_oid,
            new_head_oid: nh_oid,
            db: self,
        })
    }
}

pub struct DbTransaction {
    uuid: u128,
    old_head_oid: git2::Oid,
    new_head_oid: git2::Oid,
    db: Db,
}

impl Transaction<Db> for DbTransaction {
    fn put<V>(&mut self, value: V) -> Result<Option<V>>
    where
        V: Durable,
    {
        self.db.put(value)
    }

    fn get<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable,
    {
        self.db.get(key)
    }

    fn delete<V>(&mut self, key: &str) -> Result<V>
    where
        V: Durable,
    {
        self.db.delete(key)
    }

    fn iter<V>(&mut self) -> Result<Box<dyn Iterator<Item = Result<V>>>>
    where
        V: 'static + Durable,
    {
        self.db.iter()
    }

    fn iter_journal(
        &mut self,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Box<dyn Iterator<Item = Result<types::JournalEntry>>>> {
        self.db.iter_journal(from, to)
    }

    fn end(mut self) -> Result<Db> {
        {
            let object = err_at!(
                IOError,
                self.db
                    .repo
                    .as_ref()
                    .unwrap()
                    .find_object(self.old_head_oid, None),
                format!("git find_object")
            )?;
            err_at!(
                IOError,
                self.db
                    .repo
                    .as_ref()
                    .unwrap()
                    .reset(&object, git2::ResetType::Mixed, None),
                format!("git reset")
            )?;
            trace!(
                "git undo-txn-commit {}<-{}",
                self.old_head_oid,
                self.new_head_oid
            );
        }

        self.db.w.set_txn_uuid(0);
        self.db.put(self.db.w.clone())?;

        Ok(self.db)
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
        let dir = &self.0;
        let es = err_at!(IOError, fs::read_dir(dir), format!("{:?}", dir))?;
        for item in es {
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
    iter: JournalYears<types::JournalEntry>,
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
    type Item = Result<types::JournalEntry>;

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
                    let jy = JournalYear::new(self.journal_dir.clone(), from);
                    self.year = Some(jy);
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
                    let jm = JournalMonth::new(self.year_dir.clone(), month);
                    self.month = Some(jm);
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
