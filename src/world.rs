use std::collections::HashMap;
use serde_json::Value;
use serde_repr::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct World<ActorType, ItemType, TokenType> {
    #[serde(rename="activeUsers")]
    pub active_users: Vec<String>,
    pub actors: Vec<ActorType>,
    pub items: Vec<ItemType>,
    pub scenes: Vec<Scene<TokenType>>,
}

#[derive(Serialize, Deserialize)]
pub struct Scene<TokenType> {
    // #[serde(flatten)]
    // pub document: Document,

    pub active: bool,
    pub background: Value,
    pub tokens: Vec<TokenType>,
}

#[derive(Serialize, Deserialize)]
pub struct Document {
    #[serde(rename="_id")]
    pub id: Option<String>,
    pub img: Option<String>,
    pub name: String,
    pub flags: Value,
    pub folder: Option<String>,
    pub ownership: OwnershipMap,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(i8)]
pub enum Permissions {
    Inherit = -1,
    None = 0,
    Limited = 1,
    Observer = 2,
    Owner = 3
}

#[derive(Serialize, Deserialize)]
pub struct OwnershipMap {
    /// Default permission level
    pub default: Permissions,
    /// Maps player id to their permission level
    #[serde(flatten)]
    pub players: HashMap<String, Permissions>
}

#[derive(Serialize, Deserialize)]
pub struct Flags {
    /// Capture all miscellaneous flag data
    #[serde(flatten)]
    pub items: HashMap<String, Value>
}

#[derive(Serialize, Deserialize)]
pub struct BaseActor<ItemType, TokenType> {
    #[serde(flatten)]
    pub document: Document,

    pub items:  Vec<ItemType>,

    #[serde(rename="prototypeToken")]
    pub prototype_token: TokenType
}

#[derive(Serialize, Deserialize)]
pub struct BaseItem {
    #[serde(flatten)]
    pub document: Document,
}

#[derive(Serialize, Deserialize)]
pub struct BaseToken {
    #[serde(flatten)]
    pub document: Document,
}
