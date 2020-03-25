use jsondata::{Json, Property};

use crate::core::{Durable, Error, Result, Tag};

#[derive(Clone)]
struct Commodity<V>
where
    V: Clone,
{
    name: String,
    value: V,
    tags: Vec<Tag>,
    notes: Vec<String>,
}

impl<V> Default for Commodity<V>
where
    V: Default + Clone,
{
    fn default() -> Commodity<V> {
        Commodity {
            name: Default::default(),
            value: Default::default(),
            tags: Default::default(),
            notes: Default::default(),
        }
    }
}

impl<V> Durable<Json> for Commodity<V>
where
    V: Default + Clone + Durable<Json>,
{
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
            Property::new("value", self.value.encode()?),
            Property::new("tags", Json::Array(tags)),
            Property::new("notes", Json::Array(notes)),
        ]);

        Ok(value)
    }

    fn decode(&mut self, from: &str) -> Result<()> {
        let value: Json = err_at!(InvalidJson, from.parse())?;

        let s = err_at!(InvalidJson, value.get("/value"))?.to_string();

        self.name = json_to_native_string!(value, "/name", "commodity-name")?;
        self.value.decode(&s);
        self.tags = {
            let tags: Vec<String> = json_to_native_string_array!(value, "/tags", "commodity-tags")?;
            tags.into_iter().map(|t| t.into()).collect()
        };
        self.notes = json_to_native_string_array!(value, "/notes", "commodity-notes")?;

        Ok(())
    }
}
