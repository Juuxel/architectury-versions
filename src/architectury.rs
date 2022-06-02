use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub trait FromJson: Sized {
    fn from_json(json: &Value) -> Option<Self>;
}

trait JsonObject {
    fn convert_get<K, T>(&self, key: K) -> Option<T>
    where
        T: FromJson,
        K: AsRef<str>;
}

impl JsonObject for Map<String, Value> {
    fn convert_get<K, T>(&self, key: K) -> Option<T>
    where
        T: FromJson,
        K: AsRef<str>,
    {
        let value = self.get(key.as_ref())?;
        T::from_json(value)
    }
}

pub struct VersionDefinition {
    /// The filter regex for versions.
    pub filter: Regex,
    /// The maven-metadata.xml url.
    pub pom: String,
}

impl FromJson for VersionDefinition {
    fn from_json(json: &Value) -> Option<Self> {
        let json = json.as_object()?;
        let filter = Regex::new(json.get("filter")?.as_str()?).ok()?;
        let pom = String::from(json.get("pom")?.as_str()?);
        Some(VersionDefinition { filter, pom })
    }
}

pub enum VersionReference {
    Definition(VersionDefinition),
    Reference(String),
}

impl FromJson for VersionReference {
    fn from_json(json: &Value) -> Option<Self> {
        if json.is_object() {
            VersionDefinition::from_json(json).map(|def| VersionReference::Definition(def))
        } else if let Some(str) = json.as_str() {
            if str.starts_with('@') {
                Some(VersionReference::Reference(str.chars().skip(1).collect()))
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct GameVersionData {
    pub stable: bool,
    pub api: VersionReference,
    pub plugin: VersionReference,
    pub loom: VersionReference,
    pub injectables: VersionReference,
}

impl FromJson for GameVersionData {
    fn from_json(json: &Value) -> Option<Self> {
        let json = json.as_object()?;
        let stable = json
            .get("stable")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let api: VersionReference = json.convert_get("api")?;
        let plugin: VersionReference = json.convert_get("plugin")?;
        let loom: VersionReference = json.convert_get("loom")?;
        let injectables: VersionReference = json.convert_get("injectables")?;
        Some(GameVersionData {
            stable,
            api,
            plugin,
            loom,
            injectables,
        })
    }
}

pub struct ArchitecturyJson {
    definitions: HashMap<String, VersionDefinition>,
    pub versions: HashMap<String, GameVersionData>,
}

impl ArchitecturyJson {
    pub fn stable(&self) -> &GameVersionData {
        self.versions.values().find(|data| data.stable).unwrap()
    }

    pub fn resolve<'a>(
        &'a self,
        version_reference: &'a VersionReference,
    ) -> Option<&'a VersionDefinition> {
        match version_reference {
            VersionReference::Definition(def) => Some(def),
            VersionReference::Reference(reference) => self.definitions.get(reference),
        }
    }
}

fn map_from_json<V>(json: &Map<String, Value>) -> Option<HashMap<String, V>>
where
    V: FromJson,
{
    let mut result: HashMap<String, V> = HashMap::new();

    for key in json.keys() {
        result.insert(key.clone(), json.convert_get(key)?);
    }

    Some(result)
}

impl FromJson for ArchitecturyJson {
    fn from_json(json: &Value) -> Option<Self> {
        let json = json.as_object()?;
        let definitions_json = json.get("definitions")?.as_object()?;
        let definitions = map_from_json::<VersionDefinition>(definitions_json)?;
        let versions_json = json.get("versions")?.as_object()?;
        let versions = map_from_json::<GameVersionData>(versions_json)?;
        Some(ArchitecturyJson {
            definitions,
            versions,
        })
    }
}
