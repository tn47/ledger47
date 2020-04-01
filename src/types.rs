use chrono::{self, Datelike};
use jsondata::{Json, JsonSerialize, Property};
use uuid;

use std::{cmp, convert::TryInto};

use crate::core::{Durable, Error, Result};

#[derive(Clone, JsonSerialize)]
pub struct Workspace {
    name: String,
}

impl From<String> for Workspace {
    fn from(name: String) -> Workspace {
        Workspace { name }
    }
}

impl From<Workspace> for String {
    fn from(w: Workspace) -> String {
        w.name
    }
}

impl Default for Workspace {
    fn default() -> Workspace {
        Workspace {
            name: Default::default(),
        }
    }
}

impl Workspace {
    fn new(name: String) -> Workspace {
        Workspace { name }
    }
}

impl Durable<Json> for Workspace {
    fn to_type(&self) -> String {
        "workspace".to_string()
    }

    fn to_key(&self) -> String {
        self.to_type()
    }

    fn encode(&self) -> Result<Json> {
        Ok(err_at!(ConvertFail, self.clone().try_into())?)
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
            tags: Default::default(),
            note: Default::default(),
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

impl Durable<Json> for Commodity {
    fn to_type(&self) -> String {
        "commodity".to_string()
    }

    fn to_key(&self) -> String {
        format!("{}-{}", self.to_type(), self.name)
    }

    fn encode(&self) -> Result<Json> {
        Ok(err_at!(ConvertFail, self.clone().try_into())?)
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
    tags: Vec<String>,
    note: String,
}

impl Default for Company {
    fn default() -> Company {
        Company {
            name: Default::default(),
            created: chrono::Utc::now(),
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
            tags: Default::default(),
            note: Default::default(),
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

impl Durable<Json> for Company {
    fn to_type(&self) -> String {
        "company".to_string()
    }

    fn to_key(&self) -> String {
        format!("{}-{}", self.to_type(), self.name)
    }

    fn encode(&self) -> Result<Json> {
        Ok(err_at!(ConvertFail, self.clone().try_into())?)
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
    tags: Vec<String>,
    note: String,
}

impl Default for Ledger {
    fn default() -> Ledger {
        Ledger {
            name: Default::default(),
            created: chrono::Utc::now(),
            tags: Default::default(),
            note: Default::default(),
        }
    }
}

impl Ledger {
    fn new(name: String, created: chrono::DateTime<chrono::Utc>) -> Ledger {
        Ledger {
            name,
            created,
            tags: Default::default(),
            note: Default::default(),
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

impl Durable<Json> for Ledger {
    fn to_type(&self) -> String {
        "ledger".to_string()
    }

    fn to_key(&self) -> String {
        format!("{}-{}", self.to_type(), self.name)
    }

    fn encode(&self) -> Result<Json> {
        Ok(err_at!(ConvertFail, self.clone().try_into())?)
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let jval: Json = err_at!(InvalidJson, from.parse())?;
        *self = err_at!(InvalidJson, jval.try_into())?;
        Ok(())
    }
}

#[derive(Clone, JsonSerialize)]
pub(crate) struct Creditor {
    ledger: String,
    commodity: (String, f64),
}

#[derive(Clone, JsonSerialize)]
pub(crate) struct Debitor {
    ledger: String,
    commodity: (String, f64),
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

    fn add_creditor(&mut self, ledger: String, commodity: (String, f64)) {
        self.creditors.push(Creditor { ledger, commodity });
    }

    fn add_debitor(&mut self, ledger: String, commodity: (String, f64)) {
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

impl Durable<Json> for Transaction {
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

    fn encode(&self) -> Result<Json> {
        Ok(err_at!(ConvertFail, self.clone().try_into())?)
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let jval: Json = err_at!(InvalidJson, from.parse())?;
        *self = err_at!(InvalidJson, jval.try_into())?;
        Ok(())
    }
}
