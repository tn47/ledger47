use chrono;
use jsondata::{Json, Property};
use uuid;

use crate::core::{Durable, Error, Result, Tag};

struct Creditor {
    account: Ledger,
    commodity: Commodity,
}

struct Debitor {
    account: Ledger,
    commodity: Commodity,
}

struct Transaction {
    payee: String,
    created: chrono::DateTime<chrono::Utc>,
    creditors: Vec<Creditor>,
    debitors: Vec<Debitor>,
    tags: Vec<Tag>,
    notes: Vec<String>,
    comments: Vec<String>,
}

impl Default for Transaction {
    fn default() -> Transaction {
        Transaction {
            name: Default::default(),
            created: chrono::Utc::now(),
            creditors: Default::default(),
            debitors: Default::default(),
            tags: Default::default(),
            notes: Default::default(),
            comments: Default::default(),
        }
    }
}

impl Durable<Json> for Transaction {
    fn to_type(&self) -> String {
        "transaction".to_string()
    }

    fn to_key(&self) -> String {
        let mut key = self.to_type();
        uuid::Uuid::new_v4()
        key.push_str(&format!("-{}", self.name));
        key
    }

    fn encode(&self) -> Result<Json> {
        let aliases: Vec<Json> = native_to_json_string_array!(self.aliases.clone());
        let tags: Vec<Json> = native_to_json_string_array!(self.tags.clone());
        let notes: Vec<Json> = native_to_json_string_array!(self.notes.clone());
        let comments: Vec<Json> = native_to_json_string_array!(self.comments.clone());

        let value = Json::Object(vec![
            Property::new("name", Json::String(self.name.clone())),
            Property::new("created", Json::String(self.created.to_string())),
            Property::new("aliases", Json::Array(aliases)),
            Property::new("tags", Json::Array(tags)),
            Property::new("notes", Json::Array(notes)),
            Property::new("comments", Json::Array(comments)),
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
        self.aliases = json_to_native_string_array!(value, "/aliases", "ledger-aliases")?;
        self.tags = {
            let tags: Vec<String> = json_to_native_string_array!(value, "/tags", "ledger-tags")?;
            tags.into_iter().map(|t| t.into()).collect()
        };
        self.notes = json_to_native_string_array!(value, "/notes", "ledger-notes")?;
        self.comments = json_to_native_string_array!(value, "/comments", "ledger-comments")?;

        Ok(())
    }
}
