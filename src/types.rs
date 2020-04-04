use chrono::{self, Datelike};
use jsondata::{Json, JsonSerialize};
use uuid;

use std::{cmp, convert::TryInto};

use crate::core::{Durable, Error, Result, Store};

pub type Key = String;

struct Group {
    name: String,
}

impl From<String> for Group {
    fn from(name: String) -> Group {
        Group { name }
    }
}

impl From<Group> for String {
    fn from(g: Group) -> String {
        g.name
    }
}

#[derive(Clone, JsonSerialize)]
pub struct Workspace {
    name: String,
    commodity: Key,
}

impl Default for Workspace {
    fn default() -> Workspace {
        Workspace {
            name: Default::default(),
            commodity: Default::default(),
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

    fn to_name(&self) -> String {
        self.name.to_string()
    }

    fn to_commodity<S>(&self, store: S) -> Result<Commodity>
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

impl From<(String, f64)> for Commodity {
    fn from((name, value): (String, f64)) -> Commodity {
        Commodity::new(name, value)
    }
}

impl From<Commodity> for (String, f64) {
    fn from(c: Commodity) -> (String, f64) {
        (c.name, c.value)
    }
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
            aliases: Default::default(),
            tags: Default::default(),
            note: Default::default(),
        }
    }
}

impl Ledger {
    fn new(name: String, created: chrono::DateTime<chrono::Utc>, company: Key) -> Ledger {
        Ledger {
            name,
            created,
            company,
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

#[derive(Clone, JsonSerialize)]
pub(crate) struct Debitor {
    ledger: Key,
    commodity: (Key, f64),
}

#[derive(Clone, JsonSerialize)]
pub struct Transaction {
    pub(crate) uuid: u128,
    pub(crate) payee: String,
    #[json(to_string)]
    pub(crate) created: chrono::DateTime<chrono::Utc>,
    pub(crate) creditors: Vec<Creditor>,
    pub(crate) debitors: Vec<Debitor>,
    pub(crate) tags: Vec<String>,
    pub(crate) note: String,
}

impl Eq for Transaction {}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        self.created.partial_cmp(&rhs.created)
    }
}

impl PartialEq for Transaction {
    fn eq(&self, rhs: &Self) -> bool {
        self.created.eq(&rhs.created)
    }
}

impl Ord for Transaction {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        self.created.cmp(&rhs.created)
    }
}

impl Default for Transaction {
    fn default() -> Transaction {
        Transaction {
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

impl Transaction {
    fn new(payee: String, created: chrono::DateTime<chrono::Utc>) -> Transaction {
        Transaction {
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

impl Durable for Transaction {
    fn to_type(&self) -> String {
        "transaction".to_string()
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
