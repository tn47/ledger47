use chrono::{self, Datelike};
use jsondata::{Json, JsonSerialize};
use uuid;

use std::{
    cmp,
    convert::{TryFrom, TryInto},
    fmt, result,
};

use crate::{
    core::{Durable, Error, Result, Store},
    util,
};

pub type Key = String;

#[derive(Clone, JsonSerialize)]
pub struct Workspace {
    pub name: String,
    pub commodity: Key,
    pub remotes: Vec<String>,
    pub txn_uuid: u128,
}

// TryFrom<(name, commodity-key, remotes)>
impl TryFrom<(String, String, String)> for Workspace {
    type Error = Error;

    fn try_from((name, commodity_key, remotes): (String, String, String)) -> Result<Workspace> {
        let name = {
            let name = name.trim().to_string();
            if util::str_as_anuh(name.as_str()) == false {
                Err(Error::InvalidInput("name".to_string()))
            } else {
                Ok(name)
            }
        }?;
        let commodity = {
            let commodity = commodity_key.trim().to_string();
            if util::str_as_anuh(commodity.as_str()) == false {
                Err(Error::InvalidInput("commodity".to_string()))
            } else {
                Ok(commodity)
            }
        }?;
        let remotes: Vec<String> = {
            let err = Error::InvalidInput("remotes".to_string());
            util::csv::<String>(remotes.trim().to_string())
                .map_err(|_| err.clone())?
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect()
        };

        Ok(Workspace {
            name,
            commodity,
            remotes,
            txn_uuid: Default::default(),
        })
    }
}

impl Default for Workspace {
    fn default() -> Workspace {
        Workspace {
            name: Default::default(),
            commodity: Default::default(),
            remotes: Default::default(),
            txn_uuid: Default::default(),
        }
    }
}

impl Workspace {
    pub fn new(name: String) -> Workspace {
        let mut w: Workspace = Default::default();
        w.name = name;
        w
    }

    pub fn set_commodity(mut self, commodity: Key) -> Self {
        self.commodity = commodity;
        self
    }

    pub fn add_remote(&mut self, remote: String) -> &mut Self {
        self.remotes.push(remote);
        self
    }

    pub fn set_txn_uuid(&mut self, uuid: u128) -> &mut Self {
        self.txn_uuid = uuid;
        self
    }
}

impl Workspace {
    fn to_name(&self) -> String {
        self.name.to_string()
    }

    fn to_commodity<S>(&mut self, store: &mut S) -> Result<Commodity>
    where
        S: Store,
    {
        store.get(&self.commodity)
    }

    fn to_txn_uuid(&mut self) -> u128 {
        self.txn_uuid
    }
}

impl Durable for Workspace {
    fn to_type(&self) -> String {
        "workspace".to_string()
    }

    fn to_key(&self) -> String {
        self.to_type()
    }

    fn encode(&self) -> Result<String> {
        let jval: Json = err_at!(ConvertFail, self.clone().try_into())?;
        Ok(jval.to_string())
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let jval: Json = err_at!(InvalidJson, from.parse())?;
        *self = err_at!(InvalidJson, jval.try_into())?;
        Ok(())
    }
}

#[derive(Clone)]
struct KeyCommodity(String);

// (commodity-name,)
impl From<(String,)> for KeyCommodity {
    fn from((name,): (String,)) -> KeyCommodity {
        KeyCommodity(format!("commodity-{}", name))
    }
}

impl From<KeyCommodity> for (String,) {
    fn from(ck: KeyCommodity) -> (String,) {
        match ck.0.split('-').collect::<Vec<&str>>().as_slice() {
            ["commodity", name] => (name.to_string(),),
            _ => unreachable!(),
        }
    }
}

impl From<KeyCommodity> for Json {
    fn from(ck: KeyCommodity) -> Json {
        Json::String(ck.0)
    }
}

impl From<Json> for KeyCommodity {
    fn from(jval: Json) -> KeyCommodity {
        KeyCommodity(jval.to_string())
    }
}

impl fmt::Display for KeyCommodity {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, JsonSerialize)]
pub struct Commodity {
    name: String,
    value: f64,
    symbol: String,
    aliases: Vec<String>,
    tags: Vec<String>,
    note: String,
}

impl Default for Commodity {
    fn default() -> Commodity {
        Commodity {
            name: Default::default(),
            value: Default::default(),
            symbol: Default::default(),
            aliases: Default::default(),
            tags: Default::default(),
            note: Default::default(),
        }
    }
}

impl From<Commodity> for (String, f64) {
    fn from(c: Commodity) -> (String, f64) {
        (c.name, c.value)
    }
}

// From<(name, value)>
impl From<(String, f64)> for Commodity {
    fn from((name, value): (String, f64)) -> Commodity {
        Commodity::new(name, value)
    }
}

// TryFrom<(name, symbol, aliases, tags, note)>
impl TryFrom<(String, String, String, String, String)> for Commodity {
    type Error = Error;

    fn try_from(
        (name, symbol, aliases, tags, note): (String, String, String, String, String),
    ) -> Result<Commodity> {
        let name = {
            let name = name.trim().to_string();
            if util::str_as_anuh(name.as_str()) == false {
                Err(Error::InvalidInput("name".to_string()))
            } else {
                Ok(name)
            }
        }?;
        let symbol = symbol.trim().to_string();
        let aliases = {
            let err = Error::InvalidInput("aliases".to_string());
            let aliases: Vec<String> = util::csv::<String>(aliases.trim().to_string())
                .map_err(|_| err.clone())?
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
            for alias in aliases.iter() {
                let alias = alias.trim().to_string();
                if util::str_as_anuhdc(alias.as_str()) == false {
                    return Err(err);
                }
            }
            aliases
        };
        let tags = {
            let err = Error::InvalidInput("tags".to_string());
            let tags: Vec<String> = util::csv::<String>(tags.trim().to_string())
                .map_err(|_| err.clone())?
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
            for tag in tags.iter() {
                let tag = tag.trim().to_string();
                if util::str_as_anuhdc(tag.as_str()) == false {
                    return Err(err);
                }
            }
            tags
        };

        Ok(Commodity {
            name,
            value: Default::default(),
            symbol,
            aliases,
            tags,
            note,
        })
    }
}

impl Commodity {
    fn new(name: String, value: f64) -> Commodity {
        Commodity {
            name,
            value,
            symbol: Default::default(),
            aliases: Default::default(),
            tags: Default::default(),
            note: Default::default(),
        }
    }
}

impl Durable for Commodity {
    fn to_type(&self) -> String {
        "commodity".to_string()
    }

    fn to_key(&self) -> String {
        let ck: KeyCommodity = (self.name.clone(),).into();
        ck.to_string()
    }

    fn encode(&self) -> Result<String> {
        let jval: Json = err_at!(ConvertFail, self.clone().try_into())?;
        Ok(jval.to_string())
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let jval: Json = err_at!(InvalidJson, from.parse())?;
        *self = err_at!(InvalidJson, jval.try_into())?;
        Ok(())
    }
}

#[derive(Clone)]
struct KeyCompany(String);

// (company-name,)
impl From<(String,)> for KeyCompany {
    fn from((name,): (String,)) -> KeyCompany {
        KeyCompany(format!("company-{}", name))
    }
}

impl From<KeyCompany> for (String,) {
    fn from(ck: KeyCompany) -> (String,) {
        match ck.0.split('-').collect::<Vec<&str>>().as_slice() {
            ["company", name] => (name.to_string(),),
            _ => unreachable!(),
        }
    }
}

impl From<KeyCompany> for Json {
    fn from(ck: KeyCompany) -> Json {
        Json::String(ck.0)
    }
}

impl From<Json> for KeyCompany {
    fn from(jval: Json) -> KeyCompany {
        KeyCompany(jval.to_string())
    }
}

impl fmt::Display for KeyCompany {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, JsonSerialize)]
pub struct Company {
    name: String,
    #[json(to_string)]
    created: chrono::DateTime<chrono::Utc>,
    aliases: Vec<String>,
    tags: Vec<String>,
    note: String,
}

impl Default for Company {
    fn default() -> Company {
        Company {
            name: Default::default(),
            created: chrono::Utc::now(),
            aliases: Default::default(),
            tags: Default::default(),
            note: Default::default(),
        }
    }
}

// TryFrom<(name, created, aliases, tags, note)>
impl TryFrom<(String, String, String, String, String)> for Company {
    type Error = Error;

    fn try_from(
        (name, created, aliases, tags, note): (String, String, String, String, String),
    ) -> Result<Company> {
        let name = {
            let name = name.trim().to_string();
            if util::str_as_anuh(name.as_str()) == false {
                Err(Error::InvalidInput("name".to_string()))
            } else {
                Ok(name)
            }
        }?;
        let created: chrono::DateTime<chrono::Utc> = {
            let created = created.trim().to_string();
            match created.parse() {
                Ok(created) => Ok(created),
                Err(_) => Err(Error::InvalidInput("created".to_string())),
            }
        }?;
        let aliases = {
            let err = Error::InvalidInput("aliases".to_string());
            let aliases: Vec<String> = util::csv::<String>(aliases.trim().to_string())
                .map_err(|_| err.clone())?
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
            for alias in aliases.iter() {
                let alias = alias.trim().to_string();
                if util::str_as_anuhdc(alias.as_str()) == false {
                    return Err(err);
                }
            }
            aliases
        };
        let tags = {
            let err = Error::InvalidInput("tags".to_string());
            let tags: Vec<String> = util::csv::<String>(tags.trim().to_string())
                .map_err(|_| err.clone())?
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
            for tag in tags.iter() {
                let tag = tag.trim().to_string();
                if util::str_as_anuhdc(tag.as_str()) == false {
                    return Err(err);
                }
            }
            tags
        };

        Ok(Company {
            name,
            created,
            aliases,
            tags,
            note,
        })
    }
}

impl Company {
    fn new(name: String, created: chrono::DateTime<chrono::Utc>) -> Company {
        Company {
            name,
            created,
            aliases: Default::default(),
            tags: Default::default(),
            note: Default::default(),
        }
    }
}

impl Durable for Company {
    fn to_type(&self) -> String {
        "company".to_string()
    }

    fn to_key(&self) -> String {
        let ck: KeyCompany = (self.name.clone(),).into();
        ck.to_string()
    }

    fn encode(&self) -> Result<String> {
        let jval: Json = err_at!(ConvertFail, self.clone().try_into())?;
        Ok(jval.to_string())
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let jval: Json = err_at!(InvalidJson, from.parse())?;
        *self = err_at!(InvalidJson, jval.try_into())?;
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct KeyLedger(String);

// (company-name, ledger-name)
impl From<(String, String)> for KeyLedger {
    fn from((cname, lname): (String, String)) -> KeyLedger {
        KeyLedger(format!("ledger-{}-{}", cname, lname))
    }
}

impl From<KeyLedger> for (String, String) {
    fn from(ck: KeyLedger) -> (String, String) {
        match ck.0.split('-').collect::<Vec<&str>>().as_slice() {
            ["ledger", cname, lname] => (cname.to_string(), lname.to_string()),
            _ => unreachable!(),
        }
    }
}

impl From<KeyLedger> for Json {
    fn from(ck: KeyLedger) -> Json {
        Json::String(ck.0)
    }
}

impl From<Json> for KeyLedger {
    fn from(jval: Json) -> KeyLedger {
        KeyLedger(jval.to_string())
    }
}

impl fmt::Display for KeyLedger {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, JsonSerialize)]
pub struct Ledger {
    name: String,
    #[json(to_string)]
    created: chrono::DateTime<chrono::Utc>,
    company: Key,

    groups: Vec<String>,
    aliases: Vec<String>,
    tags: Vec<String>,
    note: String,
}

impl Default for Ledger {
    fn default() -> Ledger {
        Ledger {
            name: Default::default(),
            created: chrono::Utc::now(),
            company: Default::default(),
            groups: Default::default(),
            aliases: Default::default(),
            tags: Default::default(),
            note: Default::default(),
        }
    }
}

// TryFrom<(name, created, company-key, groups, aliases, tags, note)>
impl TryFrom<(String, String, String, String, String, String, String)> for Ledger {
    type Error = Error;

    fn try_from(
        (name, created, company_key, groups, aliases, tags, note): (
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        ),
    ) -> Result<Ledger> {
        let name = {
            let name = name.trim().to_string();
            if util::str_as_anuh(name.as_str()) == false {
                Err(Error::InvalidInput("name".to_string()))
            } else {
                Ok(name)
            }
        }?;
        let created: chrono::DateTime<chrono::Utc> = {
            let created = created.trim().to_string();
            match created.parse() {
                Ok(created) => Ok(created),
                Err(_) => Err(Error::InvalidInput("created".to_string())),
            }
        }?;
        let company = {
            let company = company_key.trim().to_string();
            if util::str_as_anuh(company.as_str()) == false {
                Err(Error::InvalidInput("company".to_string()))
            } else {
                Ok(company)
            }
        }?;
        let groups = {
            let err = Error::InvalidInput("groups".to_string());
            let groups: Vec<String> = util::csv::<String>(groups.trim().to_string())
                .map_err(|_| err.clone())?
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
            for group in groups.iter() {
                let group = group.trim().to_string();
                if util::str_as_anuhdc(group.as_str()) == false {
                    return Err(err);
                }
            }
            groups
        };
        let aliases = {
            let err = Error::InvalidInput("aliases".to_string());
            let aliases: Vec<String> = util::csv::<String>(aliases.trim().to_string())
                .map_err(|_| err.clone())?
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
            for alias in aliases.iter() {
                let alias = alias.trim().to_string();
                if util::str_as_anuhdc(alias.as_str()) == false {
                    return Err(err);
                }
            }
            aliases
        };
        let tags = {
            let err = Error::InvalidInput("tags".to_string());
            let tags: Vec<String> = util::csv::<String>(tags.trim().to_string())
                .map_err(|_| err.clone())?
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
            for tag in tags.iter() {
                let tag = tag.trim().to_string();
                if util::str_as_anuhdc(tag.as_str()) == false {
                    return Err(err);
                }
            }
            tags
        };

        Ok(Ledger {
            name,
            created,
            company,
            groups,
            aliases,
            tags,
            note,
        })
    }
}

impl Ledger {
    fn new(name: String, created: chrono::DateTime<chrono::Utc>, company: Key) -> Ledger {
        Ledger {
            name,
            created,
            company,
            groups: Default::default(),
            aliases: Default::default(),
            tags: Default::default(),
            note: Default::default(),
        }
    }
}

impl Durable for Ledger {
    fn to_type(&self) -> String {
        "ledger".to_string()
    }

    fn to_key(&self) -> String {
        let lk: KeyLedger = (self.company.clone(), self.name.clone()).into();
        lk.to_string()
    }

    fn encode(&self) -> Result<String> {
        let jval: Json = err_at!(ConvertFail, self.clone().try_into())?;
        Ok(jval.to_string())
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let jval: Json = err_at!(InvalidJson, from.parse())?;
        *self = err_at!(InvalidJson, jval.try_into())?;
        Ok(())
    }
}

#[derive(Clone, JsonSerialize)]
pub(crate) struct Creditor {
    pub(crate) ledger: KeyLedger,
    pub(crate) commodity: Commodity,
}

// (company-name, ledger-name, commodity-name, value)
impl TryFrom<(String, String, String, f64)> for Creditor {
    type Error = Error;

    fn try_from(
        (company_name, ledger_name, commodity_name, value): (String, String, String, f64),
    ) -> Result<Creditor> {
        let lk: KeyLedger = {
            let ln = ledger_name.trim().to_string();
            let ln = if util::str_as_anuh(ln.as_str()) == false {
                Err(Error::InvalidInput("ledger".to_string()))
            } else {
                Ok(ln)
            }?;
            let cn = company_name.trim().to_string();
            let cn = if util::str_as_anuh(cn.as_str()) == false {
                Err(Error::InvalidInput("company".to_string()))
            } else {
                Ok(cn)
            }?;
            (cn, ln).into()
        };

        let commodity_name = {
            let cn = commodity_name.trim().to_string();
            if util::str_as_anuh(cn.as_str()) == false {
                Err(Error::InvalidInput("commodity".to_string()))
            } else {
                Ok(cn)
            }?
        };

        Ok(Creditor {
            ledger: lk,
            commodity: (commodity_name, value).into(),
        })
    }
}

#[derive(Clone, JsonSerialize)]
pub(crate) struct Debitor {
    pub(crate) ledger: KeyLedger,
    pub(crate) commodity: Commodity,
}

// TryFrom<(company-name, ledger-name, commodity-name, value)>
impl TryFrom<(String, String, String, f64)> for Debitor {
    type Error = Error;

    fn try_from(
        (company_name, ledger_name, commodity_name, value): (String, String, String, f64),
    ) -> Result<Debitor> {
        let lk: KeyLedger = {
            let ln = ledger_name.trim().to_string();
            let ln = if util::str_as_anuh(ln.as_str()) == false {
                Err(Error::InvalidInput("ledger".to_string()))
            } else {
                Ok(ln)
            }?;
            let cn = company_name.trim().to_string();
            let cn = if util::str_as_anuh(cn.as_str()) == false {
                Err(Error::InvalidInput("company".to_string()))
            } else {
                Ok(cn)
            }?;
            (cn, ln).into()
        };

        let commodity_name = {
            let cn = commodity_name.trim().to_string();
            if util::str_as_anuh(cn.as_str()) == false {
                Err(Error::InvalidInput("commodity".to_string()))
            } else {
                Ok(cn)
            }?
        };

        Ok(Debitor {
            ledger: lk,
            commodity: (commodity_name, value).into(),
        })
    }
}

#[derive(Clone)]
struct KeyJournalEntry(String);

impl From<(i32, u32, u32, u128)> for KeyJournalEntry {
    fn from((y, m, d, uuid): (i32, u32, u32, u128)) -> KeyJournalEntry {
        KeyJournalEntry(format!("{}-{}-{}-journalentry-{}", y, m, d, uuid))
    }
}

impl From<KeyJournalEntry> for String {
    fn from(jek: KeyJournalEntry) -> String {
        jek.0
    }
}

impl From<KeyJournalEntry> for Json {
    fn from(jek: KeyJournalEntry) -> Json {
        Json::String(jek.0)
    }
}

impl From<Json> for KeyJournalEntry {
    fn from(jval: Json) -> KeyJournalEntry {
        KeyJournalEntry(jval.to_string())
    }
}

impl TryFrom<KeyJournalEntry> for (i32, u32, u32, u128) {
    type Error = Error;

    fn try_from(jek: KeyJournalEntry) -> Result<(i32, u32, u32, u128)> {
        match jek.0.split('-').collect::<Vec<&str>>().as_slice() {
            [y, m, d, uuid] => {
                let year: i32 = err_at!(ConvertFail, y.parse())?;
                let month: u32 = err_at!(ConvertFail, m.parse())?;
                let day: u32 = err_at!(ConvertFail, d.parse())?;
                let uuid: u128 = err_at!(ConvertFail, uuid.parse())?;
                Ok((year, month, day, uuid))
            }
            _ => err_at!(Fatal, msg: format!("invalid journal-entry-key {}", jek)),
        }
    }
}

impl fmt::Display for KeyJournalEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, JsonSerialize)]
pub struct JournalEntry {
    pub(crate) uuid: u128,
    pub(crate) payee: String,
    #[json(to_string)]
    pub(crate) created: chrono::DateTime<chrono::Utc>,
    pub(crate) creditors: Vec<Creditor>,
    pub(crate) debitors: Vec<Debitor>,
    pub(crate) tags: Vec<String>,
    pub(crate) note: String,
}

impl Eq for JournalEntry {}

impl PartialOrd for JournalEntry {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        self.created.partial_cmp(&rhs.created)
    }
}

impl PartialEq for JournalEntry {
    fn eq(&self, rhs: &Self) -> bool {
        self.created.eq(&rhs.created)
    }
}

impl Ord for JournalEntry {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        self.created.cmp(&rhs.created)
    }
}

impl Default for JournalEntry {
    fn default() -> JournalEntry {
        JournalEntry {
            uuid: Default::default(),
            payee: Default::default(),
            created: chrono::Utc::now(),
            creditors: Default::default(),
            debitors: Default::default(),
            tags: Default::default(),
            note: Default::default(),
        }
    }
}

// TryFrom<(uuid, payee, created, tags, note)>
impl TryFrom<(u128, String, String, String, String)> for JournalEntry {
    type Error = Error;

    fn try_from(
        (uuid, payee, created, tags, note): (u128, String, String, String, String),
    ) -> Result<JournalEntry> {
        let payee = payee.trim().to_string();
        let created: chrono::DateTime<chrono::Utc> = {
            let created = created.trim().to_string();
            match created.parse() {
                Ok(created) => Ok(created),
                Err(_) => Err(Error::InvalidInput("created".to_string())),
            }
        }?;
        let tags = {
            let err = Error::InvalidInput("tags".to_string());
            let tags: Vec<String> = util::csv::<String>(tags.trim().to_string())
                .map_err(|_| err.clone())?
                .into_iter()
                .map(|s| s.trim().to_string())
                .collect();
            for tag in tags.iter() {
                let tag = tag.trim().to_string();
                if util::str_as_anuhdc(tag.as_str()) == false {
                    return Err(err);
                }
            }
            tags
        };

        Ok(JournalEntry {
            uuid,
            payee,
            created,
            creditors: Default::default(),
            debitors: Default::default(),
            tags,
            note,
        })
    }
}

impl JournalEntry {
    fn new(payee: String, created: chrono::DateTime<chrono::Utc>) -> JournalEntry {
        JournalEntry {
            uuid: uuid::Uuid::new_v4().as_u128(),
            payee,
            created,
            creditors: Default::default(),
            debitors: Default::default(),
            tags: Default::default(),
            note: Default::default(),
        }
    }

    fn add_creditor(
        &mut self,
        company: String,
        ledger: String,
        commodity: (String, f64),
    ) -> Result<()> {
        let creditor: Creditor = (company, ledger, commodity.0, commodity.1).try_into()?;
        self.creditors.push(creditor);
        Ok(())
    }

    fn add_debitor(
        &mut self,
        company: String,
        ledger: String,
        commodity: (String, f64),
    ) -> Result<()> {
        let debitor: Debitor = (company, ledger, commodity.0, commodity.1).try_into()?;
        self.debitors.push(debitor);
        Ok(())
    }
}

impl Durable for JournalEntry {
    fn to_type(&self) -> String {
        "journalentry".to_string()
    }

    fn to_key(&self) -> String {
        let (y, m, d) = (
            self.created.year(),
            self.created.month(),
            self.created.day(),
        );
        let jek: KeyJournalEntry = (y, m, d, self.uuid).into();
        jek.into()
    }

    fn encode(&self) -> Result<String> {
        let jval: Json = err_at!(ConvertFail, self.clone().try_into())?;
        Ok(jval.to_string())
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let jval: Json = err_at!(InvalidJson, from.parse())?;
        *self = err_at!(InvalidJson, jval.try_into())?;
        Ok(())
    }
}
