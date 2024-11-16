use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "Open_Mod_Manager_Repository")]
pub struct Repository {
    pub uuid: Uuid,
    pub title: String,
    pub downpath: String,
    pub references: References,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct References {
    #[serde(rename = "@count")]
    pub count: usize,
    #[serde(default)]
    pub mods: Vec<Mod>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Mod {
    #[serde(rename = "@ident")]
    pub ident: String,
    #[serde(rename = "@file")]
    pub file: String,
    #[serde(rename = "@bytes")]
    pub bytes: usize,
    #[serde(rename = "@xxhsum")]
    pub xxhsum: String,
}
