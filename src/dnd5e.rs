#![allow(non_camel_case_types)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::world::{World, BaseActor, BaseToken, BaseItem};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DND5EActor {
    npc {
        #[serde(flatten)]
        base: BaseActor<DND5EItem, DND5EToken>,
        system: Value,
    },

    player {
        #[serde(flatten)]
        base: BaseActor<DND5EItem, DND5EToken>,
        system: Value,
    },
}

#[derive(Serialize, Deserialize, Default)]
pub struct Skills {
    #[serde(rename="acr")]
    acrobatics: Skill,

    #[serde(rename="ani")]
    animal_handling: Skill,

    #[serde(rename="arc")]
    arcana: Skill,

    #[serde(rename="ath")]
    athletics: Skill,

    #[serde(rename="dec")]
    deception: Skill,

    #[serde(rename="his")]
    history: Skill,

    #[serde(rename="ins")]
    insight: Skill,

    #[serde(rename="inv")]
    investigation: Skill,

    #[serde(rename="itm")]
    intimidation: Skill,

    #[serde(rename="med")]
    medicine: Skill,

    #[serde(rename="nature")]
    nature: Skill,

    #[serde(rename="per")]
    persuasion: Skill,

    #[serde(rename="prc")]
    perception: Skill,

    #[serde(rename="prf")]
    performance: Skill,

    #[serde(rename="rel")]
    religion: Skill,

    #[serde(rename="slt")]
    sleight_of_hand: Skill,

    #[serde(rename="ste")]
    stealth: Skill,

    #[serde(rename="sur")]
    survival: Skill,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Skill {
    ability: String,
    bonuses: SkillBonuses,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SkillBonuses {
    check: String,
    passives: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Attributes {
    ac: ArmorClass,
    hp: HitPoints
}

#[derive(Serialize, Deserialize, Default)]
pub struct ArmorClass {
    /// Observed to be "flat" or "default"
    calc: String,
    flat: Option<u8>,
    formula: Option<String>
}

#[derive(Serialize, Deserialize, Default)]
pub struct HitPoints {
    value: i32,
    max: i32,
    temp: Option<i32>,
    tempmax: Option<i32>,
    bonuses: Option<Value>,
    formula: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct Abilities {
    cha: AbilityScore,
    con: AbilityScore,
    dex: AbilityScore,
    int: AbilityScore,
    str: AbilityScore,
    wis: AbilityScore,
}

#[derive(Serialize, Deserialize)]
pub struct AbilityScore {
    value: u8,
    proficient: u8,
    bonuses: AbilityScoreBonus,
    save: String
    // Unsure what the rest means or whether it matters
}

#[derive(Serialize, Deserialize)]
pub struct AbilityScoreBonus {
    check: String,
    save: String
}


#[derive(Serialize, Deserialize)]
pub struct DND5EItem {
    #[serde(flatten)]
    base: BaseItem

}

#[derive(Serialize, Deserialize)]
pub struct DND5EToken {
    #[serde(flatten)]
    base: BaseToken
}

pub type DND5EWorld = World<DND5EActor, DND5EItem, DND5EToken>;