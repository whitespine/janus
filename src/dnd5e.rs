#![allow(non_camel_case_types)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::world::{World, BaseActor, BaseToken, BaseItem, Document};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DND5EActor {
    npc {
        #[serde(flatten)]
        base: BaseActor<DND5EItem, DND5EToken>,
        system: Value,
    },

    character {
        #[serde(flatten)]
        base: BaseActor<DND5EItem, DND5EToken>,
        system: CharacterSystem,
    },

    vehicle {
        system: Value,

    },

    group {
        system: Value,
    }
}

#[derive(Serialize, Deserialize)]
pub struct CharacterSystem {
    attributes: Attributes,
    skills: Skills
}


#[derive(Serialize, Deserialize)]
pub struct Skills {
    #[serde(rename="acr")]
    pub acrobatics: Skill,

    #[serde(rename="ani")]
    pub animal_handling: Skill,

    #[serde(rename="arc")]
    pub arcana: Skill,

    #[serde(rename="ath")]
    pub athletics: Skill,

    #[serde(rename="dec")]
    pub deception: Skill,

    #[serde(rename="his")]
    pub history: Skill,

    #[serde(rename="ins")]
    pub insight: Skill,

    #[serde(rename="inv")]
    pub investigation: Skill,

    #[serde(rename="itm")]
    pub intimidation: Skill,

    #[serde(rename="med")]
    pub medicine: Skill,

    #[serde(rename="nat")]
    pub nature: Skill,

    #[serde(rename="per")]
    pub persuasion: Skill,

    #[serde(rename="prc")]
    pub perception: Skill,

    #[serde(rename="prf")]
    pub performance: Skill,

    #[serde(rename="rel")]
    pub religion: Skill,

    #[serde(rename="slt")]
    pub sleight_of_hand: Skill,

    #[serde(rename="ste")]
    pub stealth: Skill,

    #[serde(rename="sur")]
    pub survival: Skill,
}

#[derive(Serialize, Deserialize)]
pub struct Skill {
    pub ability: String,
    pub bonuses: SkillBonuses,
}

#[derive(Serialize, Deserialize)]
pub struct SkillBonuses {
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub passives: String,
}

#[derive(Serialize, Deserialize)]
pub struct Attributes {
    pub ac: ArmorClass,
    pub hp: HitPoints
}

#[derive(Serialize, Deserialize)]
pub struct ArmorClass {
    /// Observed to be "flat" or "default"
    pub calc: String,
    pub flat: Option<u8>,
    pub formula: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct HitPoints {
    #[serde(default)]
    pub value: i32,
    pub max: Option<i32>,
    pub temp: Option<i32>,
    pub tempmax: Option<i32>,
    pub bonuses: Option<Value>,
    pub formula: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct Abilities {
    pub cha: AbilityScore,
    pub con: AbilityScore,
    pub dex: AbilityScore,
    pub int: AbilityScore,
    pub str: AbilityScore,
    pub wis: AbilityScore,
}

#[derive(Serialize, Deserialize)]
pub struct AbilityScore {
    pub value: u8,
    #[serde(default)]
    pub proficient: u8,
    #[serde(default)]
    pub bonuses: AbilityScoreBonus,
    #[serde(default)]
    pub save: String
    // Unsure what the rest means or whether it matters
}

#[derive(Serialize, Deserialize, Default)]
pub struct AbilityScoreBonus {
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub save: String
}


#[derive(Serialize, Deserialize)]
pub struct DND5EItem {
    // #[serde(flatten)]
    // pub base: BaseItem

}

#[derive(Serialize, Deserialize)]
pub struct DND5EToken {
    // #[serde(flatten)]
    // pub base: BaseToken
}

pub type DND5EWorld = World<DND5EActor, DND5EItem, DND5EToken>;