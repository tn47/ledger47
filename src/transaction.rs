use chrono;
use jsondata::{Json, Property};
use uuid;

use crate::core::{Durable, Error, Result};

#[derive(Clone)]
struct Creditor {
    ledger: String,
    commodity: (String, f64),
}

#[derive(Clone)]
struct Debitor {
    ledger: String,
    commodity: (String, f64),
}

#[derive(Clone)]
struct Transaction {
    uuid: u128,
    payee: String,
    created: chrono::DateTime<chrono::Utc>,
    creditors: Vec<Creditor>,
    debitors: Vec<Debitor>,
    tags: Vec<String>,
    note: String,
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
        let mut key = self.to_type();
        key.push_str(&format!("-{}", self.uuid));
        key
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
