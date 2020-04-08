use chrono::{self, Datelike};
use jsondata::{Json, JsonSerialize};
use uuid;

use std::{
    cmp,
    convert::{TryFrom, TryInto},
};

use crate::{
    core::{Durable, Error, Result, Store},
    util,
};

pub type Key = String;

#[derive(Clone, JsonSerialize)]
pub struct Workspace {
    name: String,
    commodity: Key,
    remotes: Vec<String>,
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
        })
    }
}

impl Default for Workspace {
    fn default() -> Workspace {
        Workspace {
            name: Default::default(),
            commodity: Default::default(),
            remotes: Default::default(),
        }
    }
}

impl Workspace {
    fn new(name: String) -> Workspace {
        let mut w: Workspace = Default::default();
        w.name = name;
        w
    }

    fn set_commodity(mut self, commodity: Key) -> Self {
        self.commodity = commodity;
        self
    }

    fn add_remote(&mut self, remote: String) -> &mut Self {
        self.remotes.push(remote);
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

    fn set_symbol(mut self, symbol: String) -> Commodity {
        self.symbol = symbol;
        self
    }

    fn has_alias(&self, alias: &str) -> bool {
        self.aliases.iter().any(|a| a == alias)
    }

    fn add_alias(&mut self, alias: &str) {
        if !self.has_alias(alias) {
            self.aliases.push(alias.to_string())
        }

        self.aliases.sort();
    }

    fn remove_alias(&mut self, alias: &str) {
        for i in 0..self.aliases.len() {
            if self.aliases[i] == alias {
                self.aliases.remove(i);
                break;
            }
        }
    }

    fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    fn add_tag(&mut self, tag: &str) {
        if !self.has_tag(tag) {
            self.tags.push(tag.to_string())
        }

        self.tags.sort();
    }

    fn remove_tag(&mut self, tag: &str) {
        for i in 0..self.tags.len() {
            if self.tags[i] == tag {
                self.tags.remove(i);
                break;
            }
        }
    }

    fn set_note(&mut self, note: String) {
        self.note = note
    }
}

impl Durable for Commodity {
    fn to_type(&self) -> String {
        "commodity".to_string()
    }

    fn to_key(&self) -> String {
        format!("{}-{}", self.to_type(), self.name)
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

    fn has_alias(&self, alias: &str) -> bool {
        self.aliases.iter().any(|a| a == alias)
    }

    fn add_alias(&mut self, alias: &str) {
        if !self.has_alias(alias) {
            self.aliases.push(alias.to_string())
        }

        self.aliases.sort();
    }

    fn remove_alias(&mut self, alias: &str) {
        for i in 0..self.aliases.len() {
            if self.aliases[i] == alias {
                self.aliases.remove(i);
                break;
            }
        }
    }

    fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    fn add_tag(&mut self, tag: &str) {
        if !self.has_tag(tag) {
            self.tags.push(tag.to_string())
        }

        self.tags.sort();
    }

    fn remove_tag(&mut self, tag: &str) {
        for i in 0..self.tags.len() {
            if self.tags[i] == tag {
                self.tags.remove(i);
                break;
            }
        }
    }

    fn set_note(&mut self, note: String) {
        self.note = note
    }
}

impl Durable for Company {
    fn to_type(&self) -> String {
        "company".to_string()
    }

    fn to_key(&self) -> String {
        format!("{}-{}", self.to_type(), self.name)
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

    fn has_alias(&self, alias: &str) -> bool {
        self.aliases.iter().any(|a| a == alias)
    }

    fn add_alias(&mut self, alias: &str) {
        if !self.has_alias(alias) {
            self.aliases.push(alias.to_string())
        }

        self.aliases.sort();
    }

    fn remove_alias(&mut self, alias: &str) {
        for i in 0..self.aliases.len() {
            if self.aliases[i] == alias {
                self.aliases.remove(i);
                break;
            }
        }
    }

    fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    fn add_tag(&mut self, tag: &str) {
        if !self.has_tag(tag) {
            self.tags.push(tag.to_string())
        }

        self.tags.sort();
    }

    fn remove_tag(&mut self, tag: &str) {
        for i in 0..self.tags.len() {
            if self.tags[i] == tag {
                self.tags.remove(i);
                break;
            }
        }
    }

    fn set_note(&mut self, note: String) {
        self.note = note
    }
}

impl Durable for Ledger {
    fn to_type(&self) -> String {
        "ledger".to_string()
    }

    fn to_key(&self) -> String {
        format!("{}-{}-{}", self.to_type(), self.company, self.name)
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
    ledger: Key,
    commodity: (Key, f64),
}

// TryFrom<(ledger-key, commodity-key)>
impl TryFrom<(String, String, f64)> for Creditor {
    type Error = Error;

    fn try_from((ledger_key, commodity_key, value): (String, String, f64)) -> Result<Creditor> {
        let ledger = {
            let ledger_key = ledger_key.trim().to_string();
            if util::str_as_anuh(ledger_key.as_str()) == false {
                Err(Error::InvalidInput("ledger".to_string()))
            } else {
                Ok(ledger_key)
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

        Ok(Creditor {
            ledger,
            commodity: (commodity, value).into(),
        })
    }
}
#[derive(Clone, JsonSerialize)]
pub(crate) struct Debitor {
    ledger: Key,
    commodity: (Key, f64),
}

// TryFrom<(ledger-key, commodity-key)>
impl TryFrom<(String, String, f64)> for Debitor {
    type Error = Error;

    fn try_from((ledger_key, commodity_key, value): (String, String, f64)) -> Result<Debitor> {
        let ledger = {
            let ledger_key = ledger_key.trim().to_string();
            if util::str_as_anuh(ledger_key.as_str()) == false {
                Err(Error::InvalidInput("ledger".to_string()))
            } else {
                Ok(ledger_key)
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

        Ok(Debitor {
            ledger,
            commodity: (commodity, value).into(),
        })
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

// TryFrom<(uuid, payee, created, creditors, debitors, tags, note)>
impl
    TryFrom<(
        u128,
        String,
        String,
        Vec<Creditor>,
        Vec<Debitor>,
        String,
        String,
    )> for JournalEntry
{
    type Error = Error;

    fn try_from(
        (uuid, payee, created, creditors, debitors, tags, note): (
            u128,
            String,
            String,
            Vec<Creditor>,
            Vec<Debitor>,
            String,
            String,
        ),
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
            creditors,
            debitors,
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

    fn add_creditor(&mut self, ledger: Key, commodity: (Key, f64)) {
        self.creditors.push(Creditor { ledger, commodity });
    }

    fn add_debitor(&mut self, ledger: Key, commodity: (Key, f64)) {
        self.debitors.push(Debitor { ledger, commodity });
    }

    fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    fn add_tag(&mut self, tag: &str) {
        if !self.has_tag(tag) {
            self.tags.push(tag.to_string())
        }

        self.tags.sort();
    }

    fn remove_tag(&mut self, tag: &str) {
        for i in 0..self.tags.len() {
            if self.tags[i] == tag {
                self.tags.remove(i);
                break;
            }
        }
    }

    fn set_note(&mut self, note: String) {
        self.note = note
    }
}

impl Durable for JournalEntry {
    fn to_type(&self) -> String {
        "journalentry".to_string()
    }

    fn to_key(&self) -> String {
        format!(
            "{}-{}-{}-{}-{}",
            self.created.year(),
            self.created.month(),
            self.created.day(),
            self.to_type(),
            self.uuid
        )
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
