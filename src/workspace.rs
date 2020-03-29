use jsondata::{Json, Property};

use crate::core::{Durable, Error, Result};

#[derive(Clone)]
struct Workspace {
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
