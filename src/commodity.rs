use jsondata::{Json, Property};

use crate::core::{Durable, Error, Result, Tag};

#[derive(Clone)]
struct Commodity {
    name: String,
    value: f64,
    tags: Vec<Tag>,
    notes: Vec<String>,
}

impl Default for Commodity {
    fn default() -> Commodity {
        Commodity {
            name: Default::default(),
            value: Default::default(),
            tags: Default::default(),
            notes: Default::default(),
        }
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
        let notes: Vec<Json> = native_to_json_string_array!(self.notes.clone());

        let value = Json::Object(vec![
            Property::new("name", Json::String(self.name.clone())),
            Property::new("value", Json::new(self.value)),
            Property::new("tags", Json::Array(tags)),
            Property::new("notes", Json::Array(notes)),
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
        self.tags = {
            let tags: Vec<String> = json_to_native_string_array!(value, "/tags", "commodity-tags")?;
            tags.into_iter().map(|t| t.into()).collect()
        };
        self.notes = json_to_native_string_array!(value, "/notes", "commodity-notes")?;

        Ok(())
    }
}
