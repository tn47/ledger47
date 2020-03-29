use chrono::{self, Datelike};
use jsondata::{Json, Property};
use uuid;

use std::cmp;

use crate::core::{Durable, Error, Result};

#[derive(Clone)]
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
        let value = Json::Object(vec![Property::new("name", Json::String(self.name.clone()))]);

        Ok(value)
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let value: Json = err_at!(InvalidJson, from.parse())?;

        self.name = json_to_native_string!(value, "/name", "workspace-name")?;

        Ok(())
    }
}

#[derive(Clone)]
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
        let tags: Vec<Json> = native_to_json_string_array!(self.tags.clone());

        let value = Json::Object(vec![
            Property::new("name", Json::String(self.name.clone())),
            Property::new("value", Json::new(self.value)),
            Property::new("tags", Json::Array(tags)),
            Property::new("note", Json::String(self.note.clone())),
        ]);

        Ok(value)
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let value: Json = err_at!(InvalidJson, from.parse())?;

        self.name = json_to_native_string!(value, "/name", "commodity-name")?;
        self.value = match err_at!(InvalidJson, value.get("/value"))?.float() {
            Some(f) => f,
            None => err_at!(InvalidJson, msg: format!("expected float"))?,
        };
        self.tags = json_to_native_string_array!(value, "/tags", "commodity-tags")?;
        self.note = json_to_native_string!(value, "/note", "commodity-note")?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct Company {
    // required fields
    name: String,
    created: chrono::DateTime<chrono::Utc>,
    // optional fields
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
        let tags: Vec<Json> = native_to_json_string_array!(self.tags.clone());

        let value = Json::Object(vec![
            Property::new("name", Json::String(self.name.clone())),
            Property::new("created", Json::String(self.created.to_string())),
            Property::new("tags", Json::Array(tags)),
            Property::new("note", Json::String(self.note.clone())),
        ]);

        Ok(value)
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let value: Json = err_at!(InvalidJson, from.parse())?;

        self.name = json_to_native_string!(value, "/name", "company-name")?;
        self.created = {
            let created = json_to_native_string!(value, "/created", "company-created")?;
            err_at!(InvalidJson, created.parse())?
        };
        self.tags = json_to_native_string_array!(value, "/tags", "company-tags")?;
        self.note = json_to_native_string!(value, "/note", "company-note")?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct Ledger {
    name: String,
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
        let tags: Vec<Json> = native_to_json_string_array!(self.tags.clone());

        let value = Json::Object(vec![
            Property::new("name", Json::String(self.name.clone())),
            Property::new("created", Json::String(self.created.to_string())),
            Property::new("tags", Json::Array(tags)),
            Property::new("note", Json::String(self.note.clone())),
        ]);

        Ok(value)
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let value: Json = err_at!(InvalidJson, from.parse())?;

        self.name = json_to_native_string!(value, "/name", "ledger-name")?;
        self.created = {
            let created = json_to_native_string!(value, "/created", "ledger-created")?;
            err_at!(InvalidJson, created.parse())?
        };
        self.tags = json_to_native_string_array!(value, "/tags", "ledger-tags")?;
        self.note = json_to_native_string!(value, "/note", "ledger-note")?;

        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct Creditor {
    ledger: String,
    commodity: (String, f64),
}

#[derive(Clone)]
pub(crate) struct Debitor {
    ledger: String,
    commodity: (String, f64),
}

#[derive(Clone)]
pub struct Transaction {
    pub(crate) uuid: u128,
    pub(crate) payee: String,
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
        let tags: Vec<Json> = native_to_json_string_array!(self.tags.clone());

        let mut creditors: Vec<Json> = vec![];
        for creditor in self.creditors.iter() {
            let commodity = Json::Object(vec![
                Property::new("name", Json::new(creditor.commodity.0.clone())),
                Property::new("value", Json::new(creditor.commodity.1)),
            ]);
            creditors.push(Json::Object(vec![
                Property::new("ledger", Json::String(creditor.ledger.clone())),
                Property::new("commodity", commodity),
            ]));
        }

        let mut debitors: Vec<Json> = vec![];
        for debitor in self.debitors.iter() {
            let commodity = Json::Object(vec![
                Property::new("name", Json::new(debitor.commodity.0.clone())),
                Property::new("value", Json::new(debitor.commodity.1)),
            ]);
            debitors.push(Json::Object(vec![
                Property::new("ledger", Json::String(debitor.ledger.clone())),
                Property::new("commodity", commodity),
            ]));
        }

        let value = Json::Object(vec![
            Property::new("uuid", Json::String(self.uuid.to_string())),
            Property::new("payee", Json::String(self.payee.clone())),
            Property::new("created", Json::String(self.created.to_string())),
            Property::new("creditors", Json::Array(creditors)),
            Property::new("debitors", Json::Array(debitors)),
            Property::new("tags", Json::Array(tags)),
            Property::new("note", Json::String(self.note.clone())),
        ]);

        Ok(value)
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let value: Json = err_at!(InvalidJson, from.parse())?;

        self.uuid = {
            let uuid = json_to_native_string!(value, "/uuid", "transaction-uuid")?;
            err_at!(InvalidJson, uuid.parse())?
        };
        self.payee = json_to_native_string!(value, "/payee", "transaction-name")?;
        self.created = {
            let created = json_to_native_string!(value, "/created", "transaction-created")?;
            err_at!(InvalidJson, created.parse())?
        };

        match err_at!(InvalidJson, value.get("/creditors"))?.array() {
            Some(cs) => {
                for c in cs.into_iter() {
                    let ledger = json_to_native_string!(c, "/ledger", "transaction-creditor")?;
                    let name =
                        json_to_native_string!(c, "/commodity/name", "transaction-creditor")?;
                    let value = err_at!(InvalidJson, c.get("/commodity/value"))?
                        .float()
                        .unwrap();
                    self.creditors.push(Creditor {
                        ledger,
                        commodity: (name, value),
                    });
                }
            }
            None => return Err(Error::InvalidJson(format!("transaction-creditors"))),
        }
        match err_at!(InvalidJson, value.get("/debitors"))?.array() {
            Some(ds) => {
                for d in ds.into_iter() {
                    let ledger = json_to_native_string!(d, "/ledger", "transaction-creditor")?;
                    let name =
                        json_to_native_string!(d, "/commodity/name", "transaction-creditor")?;
                    let value = err_at!(InvalidJson, d.get("/commodity/value"))?
                        .float()
                        .unwrap();
                    self.debitors.push(Debitor {
                        ledger,
                        commodity: (name, value),
                    });
                }
            }
            None => return Err(Error::InvalidJson(format!("transaction-debitors"))),
        }
        self.tags = json_to_native_string_array!(value, "/tags", "transaction-tags")?;
        self.note = json_to_native_string!(value, "/note", "transaction-note")?;

        Ok(())
    }
}
