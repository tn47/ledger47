use chrono;

use crate::core::{Durable, Error, Result, Tag};

#[derive(Clone)]
struct Company {
    // required fields
    name: String,
    created: chrono::DateTime<chrono::Utc>,
    // optional fields
    aliases: Vec<String>,
    tags: Vec<Tag>,
    notes: Vec<String>,
    comments: Vec<String>,
}

impl Default for Company {
    fn default() -> Company {
        Company {
            name: Default::default(),
            created: chrono::Utc::now(),
            aliases: Default::default(),
            tags: Default::default(),
            notes: Default::default(),
            comments: Default::default(),
        }
    }
}

impl Durable for Company {
    fn to_type(&self) -> String {
        "company".to_string()
    }

    fn to_key(&self) -> String {
        let mut key = self.to_type();
        key.push_str(&format!("-{}", self.name));
        key
    }

    fn encode(&self, buffer: &mut Vec<u8>) -> Result<usize> {
        use jsondata::{Json, Property};

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

        let json_string = value.to_string();
        buffer.extend_from_slice(&json_string.as_bytes());
        Ok(json_string.as_bytes().len())
    }

    fn decode(&mut self, buffer: &[u8]) -> Result<usize> {
        use jsondata::Json;

        let s = err_at!(InvalidJson, std::str::from_utf8(buffer))?;
        let value: Json = err_at!(InvalidJson, s.parse())?;

        self.name = json_to_native_string!(value, "/name", "company-name")?;
        self.created = {
            let created = json_to_native_string!(value, "/created", "company-created")?;
            err_at!(InvalidJson, created.parse())?
        };
        self.aliases = json_to_native_string_array!(value, "/aliases", "company-aliases")?;
        self.tags = {
            let tags: Vec<String> = json_to_native_string_array!(value, "/tags", "company-tags")?;
            tags.into_iter().map(|t| t.into()).collect()
        };
        self.notes = json_to_native_string_array!(value, "/notes", "company-notes")?;
        self.comments = json_to_native_string_array!(value, "/comments", "company-comments")?;

        Ok(buffer.len())
    }
}
