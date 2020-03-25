use chrono;
use jsondata::{Json, Property};

use crate::core::{Durable, Error, Result};

#[derive(Clone)]
struct Ledger {
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
        let mut key = self.to_type();
        key.push_str(&format!("-{}", self.name));
        key
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
