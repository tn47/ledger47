use jsondata::{Json, Property};

use crate::core::{Durable, Error, Result};

#[derive(Clone)]
struct Commodity {
    name: String,
    value: f64,
    tags: Vec<String>,
    note: String,
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
        let mut key = self.to_type();
        key.push_str(&format!("-{}", self.name));
        key
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
