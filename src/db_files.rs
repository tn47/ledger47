use std::{ffi, fs, path};

use crate::core::{Durable, Error, Result};

pub struct Workspace(ffi::OsString);

impl Workspace {
    pub fn to_metadata_dir(&self) -> MetadataDir {
        let mut pp = path::PathBuf::new();
        pp.push(path::Path::new(&self.0));
        pp.push("metadata");
        MetadataDir(pp.into_os_string())
    }
}

pub struct MetadataDir(ffi::OsString);

impl MetadataDir {
    pub fn new<T: Durable>(&self, value: T) -> Result<DataFile<T>> {
        let file_loc = {
            let mut pp = path::PathBuf::new();
            pp.push(path::Path::new(&self.0));
            pp.push(&format!(
                "{}-{}.json",
                value.to_type(),
                value.to_unique_name()
            ));
            pp.into_os_string()
        };
        DataFile::new(value.to_type().as_str(), file_loc, value)
    }

    pub fn iter<T: Durable>(&self) -> Result<impl Iterator<Item = DataFile<T>>> {
        let mut data_files = vec![];
        for item in err_at!(IOError, fs::read_dir(&self.0))? {
            let item = err_at!(IOError, item)?;

            let mut value: T = Default::default();
            let typ = value.to_type();

            match item.file_name().to_str() {
                Some(file_name) if file_name.starts_with(&typ) => {
                    let file_loc = {
                        let mut pp = path::PathBuf::new();
                        pp.push(path::Path::new(&self.0));
                        pp.push(&file_name);
                        pp.into_os_string()
                    };
                    let data = err_at!(IOError, fs::read(&file_loc))?;
                    value.decode(&data)?;

                    let df = DataFile::new(&typ, file_loc, value)?;
                    data_files.push(df)
                }
                Some(_) => (),
                None => err_at!(Fatal, msg: format!("{:?}", item.file_name()))?,
            }
        }

        Ok(data_files.into_iter())
    }
}

#[derive(Clone)]
pub enum DataFile<T: Durable> {
    Company { file_loc: ffi::OsString, value: T },
    Ledger { file_loc: ffi::OsString, value: T },
    Commodity { file_loc: ffi::OsString, value: T },
    Transaction { file_loc: ffi::OsString, value: T },
}

impl<T: Durable> DataFile<T> {
    pub fn new(typ: &str, file_loc: ffi::OsString, value: T) -> Result<DataFile<T>> {
        match typ {
            "company" => Ok(DataFile::Company { file_loc, value }),
            "ledger" => Ok(DataFile::Ledger { file_loc, value }),
            "commodity" => Ok(DataFile::Commodity { file_loc, value }),
            "transaction" => Ok(DataFile::Transaction { file_loc, value }),
            _ => err_at!(Fatal, msg: format!("unreachable")),
        }
    }

    fn to_url(&self) -> ffi::OsString {
        match self {
            DataFile::Company { file_loc, .. } => file_loc.clone(),
            DataFile::Ledger { file_loc, .. } => file_loc.clone(),
            DataFile::Commodity { file_loc, .. } => file_loc.clone(),
            DataFile::Transaction { file_loc, .. } => file_loc.clone(),
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

    fn is_old(&self) -> Result<bool> {
        let file_loc = match self {
            DataFile::Company { file_loc, .. } => file_loc,
            DataFile::Ledger { file_loc, .. } => file_loc,
            DataFile::Commodity { file_loc, .. } => file_loc,
            DataFile::Transaction { file_loc, .. } => file_loc,
        };
        match file_loc.to_str() {
            Some(file_loc) => Ok(file_loc.ends_with(".old")),
            None => err_at!(Fatal, msg: format!("{:?}", file_loc)),
        }
    }

    fn older_version(&self, other: &DataFile<T>) -> Result<Option<ffi::OsString>> {
        if self.is_old()? == false && other.is_old()? == false {
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
        let mut data = vec![];
        value.encode(&mut data)?;

        let old_value = self.swap_value(value);
        let old_file_loc = {
            let mut old_file_loc = self.to_url().clone();
            old_file_loc.push(".old");
            old_file_loc
        };

        err_at!(IOError, fs::rename(&self.to_url(), &old_file_loc))?;
        err_at!(IOError, fs::write(&self.to_url(), &data))?;
        err_at!(IOError, fs::remove_file(&old_file_loc))?;

        Ok(old_value)
    }

    pub fn delete(self) -> Result<()> {
        let file_loc = match self {
            DataFile::Company { file_loc, .. } => file_loc,
            DataFile::Ledger { file_loc, .. } => file_loc,
            DataFile::Commodity { file_loc, .. } => file_loc,
            DataFile::Transaction { file_loc, .. } => file_loc,
        };
        err_at!(IOError, fs::remove_file(file_loc))
    }
}
