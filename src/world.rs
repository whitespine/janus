use std::collections::HashMap;
use serde_json::Value;
use serde_repr::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct World<ItemType, ActorType, TokenType> {
    #[serde(rename="activeUsers")]
    active_users: Vec<String>,
    actors: Vec<ActorType>,
    items: Vec<ItemType>,
    scenes: Vec<Scene<TokenType>>,
}

#[derive(Serialize, Deserialize)]
pub struct Scene<TokenType> {
    #[serde(flatten)]
    document: Document,

    active: bool,
    background: Value,
    tokens: Vec<TokenType>,
}

#[derive(Serialize, Deserialize)]
pub struct Document {
    #[serde(rename="_id")]
    id: String,
    img: String,
    name: String,
    flags: Value,
    folder: Option<String>,
    ownership: OwnershipMap,
    #[serde(rename="type")]
    data_type: String,
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
    default: Permissions,
    /// Maps player id to their permission level
    #[serde(flatten)]
    players: HashMap<String, Permissions>
}

#[derive(Serialize, Deserialize)]
pub struct Flags {
    /// Capture all miscellaneous flag data
    #[serde(flatten)]
    items: HashMap<String, Value>
}

#[derive(Serialize, Deserialize)]
pub struct BaseActor<ItemType, TokenType> {
    #[serde(flatten)]
    document: Document,

    items:  Vec<ItemType>,

    #[serde(rename="prototypeToken")]
    prototype_token: TokenType
}

#[derive(Serialize, Deserialize)]
pub struct BaseItem {
    #[serde(flatten)]
    document: Document,
}

#[derive(Serialize, Deserialize)]
pub struct BaseToken {
    #[serde(flatten)]
    document: Document,
}
