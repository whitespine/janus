mod connection;
mod dnd5e;
pub mod error;
mod world;

use crate::connection::FoundryClient;
use crate::dnd5e::{DND5EActor, DND5EItem, DND5EWorld};
use clap::Parser;
use rust_socketio::Payload;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use poise::serenity_prelude as serenity;
use std::env;
use caith::Roller;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use crate::error::CommandError;
use crate::error::CommandError::InvalidAttribute;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Url to connect to. Include http(s):// and any trailing suffix, if needed
    #[arg(long)]
    host: String,
}

fn to_one_json(payload: Payload) -> serde_json::Value {
    // if let Payload::Text(v) = payload;
    match payload {
        Payload::Text(mut items) => items.remove(0),
        _ => panic!("Not json!"),
    }
}

// Our poise types
struct DiscordState {
    foundry: FoundryClient,
    store: tokio::sync::Mutex<PickleDb>
} // User data, which is stored and accessible in all command invocations
type DiscordError = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, DiscordState, DiscordError>;




/// Associates a user with an actor
#[poise::command(slash_command)]
async fn assoc(
    ctx: Context<'_>,
    #[description = "Actor Name"] name: String,
) -> Result<(), DiscordError> {
    let foundry = &ctx.data().foundry;
    let world = get_world(foundry).await?;

    // Figure out who their user id
    let user_id = ctx.author().id.get();

    match world.actors.iter().find_map(|actor| {
        if let DND5EActor::character { base, .. } = actor {
            if base.document.name == name && base.document.id.is_some() {
                return Some(base.document.id.clone().unwrap());
            }
        }
        None
    }) {
        Some(id) => { // I clearly fucked up the typings here but... ???
            let mut store = ctx.data().store.lock().await;
            store.set(&user_id.to_string(), &id)?;
            ctx.say(format!("Successfully associated user id {} with actor id {}", user_id, id)).await?;
        },
        _ => {
            Err(CommandError::CharacterNotFound(name))?;
        },
    }

    Ok(())
}

/// Rolls a stat
#[poise::command(slash_command)]
async fn roll(
    ctx: Context<'_>,
    #[description = "Attribute"] stat: String,
    #[description = "adv/dis"] adv_or_dis: Option<String>
) -> Result<(), DiscordError> {
    let foundry = &ctx.data().foundry;
    let world = get_world(foundry).await?;

    // Figure out who they should be
    let user_id = ctx.author().id.get();
    let store = ctx.data().store.lock().await;
    let actor_id: Option<String> = store.get(&user_id.to_string());

    // See if it works
    let actor_id = actor_id.ok_or(CommandError::MissingAssocChar)?;

    // Attempt to find character system data
    let actor =  world.actors.iter().find(|actor| {
        if let DND5EActor::character { base, .. } = actor {
            return base.document.id == Some(actor_id.clone());
        }
        false
    }).ok_or(CommandError::InvalidAssocChar)?;
    let DND5EActor::character { system, base } = actor else { Err(CommandError::InvalidAssocChar)? };

    // We will need proficiency in a lot of cases
    let mut total_level = 0;
    for item in &base.items {
        match item {
            DND5EItem::class { system, .. } => { total_level += system.levels.unwrap_or(0); }
            _ => {} // Don't care
        }
    }
    let proficiency = match total_level {
        0..=4 => 2,
        5..=8 => 3,
        9..=12 => 4,
        13..=16 => 5,
        17..=20 => 6,
        _ => 7
    };

    // Get the appropriate stats
    let (ability_score, proficiency_factor) = match stat.as_str() {
        "str" | "strength"      => (system.abilities.str.value, 0f32),
        "dex" | "dexterity"     => (system.abilities.str.value, 0f32),
        "cha" | "charisma"      => (system.abilities.str.value, 0f32),
        "int" | "intelligence"  => (system.abilities.str.value, 0f32),
        "wis" | "wisdom"        => (system.abilities.str.value, 0f32),
        "con" | "constitution"  => (system.abilities.str.value, 0f32),
        "acr" | "acrobatics" => (system.abilities.dex.value, system.skills.acrobatics.value.unwrap_or(0f32)),
        "ani" | "animal" | "animals" => (system.abilities.wis.value, system.skills.animal_handling.value.unwrap_or(0f32)),
        "arc" | "arcana" => (system.abilities.int.value, system.skills.arcana.value.unwrap_or(0f32)),
        "ath" | "athletics" => (system.abilities.str.value, system.skills.athletics.value.unwrap_or(0f32)),
        "dec" | "deception" => (system.abilities.cha.value, system.skills.deception.value.unwrap_or(0f32)),
        "his" | "history" => (system.abilities.int.value, system.skills.history.value.unwrap_or(0f32)),
        "ins" | "insight" => (system.abilities.wis.value, system.skills.insight.value.unwrap_or(0f32)),
        "inv" | "investigation" => (system.abilities.int.value, system.skills.investigation.value.unwrap_or(0f32)),
        "itm" | "intimidation" => (system.abilities.cha.value, system.skills.intimidation.value.unwrap_or(0f32)),
        "med" | "medicine" => (system.abilities.wis.value, system.skills.medicine.value.unwrap_or(0f32)),
        "nat" | "nature" => (system.abilities.int.value, system.skills.nature.value.unwrap_or(0f32)),
        "per" | "persuasion" => (system.abilities.cha.value, system.skills.persuasion.value.unwrap_or(0f32)),
        "prc" | "perception" => (system.abilities.wis.value, system.skills.perception.value.unwrap_or(0f32)),
        "prf" | "performance" => (system.abilities.cha.value, system.skills.performance.value.unwrap_or(0f32)),
        "rel" | "religion" => (system.abilities.int.value, system.skills.religion.value.unwrap_or(0f32)),
        "slt" | "sleight" | "sleight of hand" => (system.abilities.dex.value, system.skills.sleight_of_hand.value.unwrap_or(0f32)),
        "ste" | "stealth" => (system.abilities.dex.value, system.skills.stealth.value.unwrap_or(0f32)),
        "sur" | "survival" => (system.abilities.wis.value, system.skills.survival.value.unwrap_or(0f32)),
        _ => return Err(InvalidAttribute(stat).into())
    };

    // Now coerce the proficiency to an integer, use a small rounding factor to ensure its more reliable
    let proficiency = ((proficiency as f32) * proficiency_factor + 0.25f32).floor() as i32;

    // Decide base roll based on adv/disadv
    let mut d20 = "1d20";
    if let Some(adv_or_dis) = adv_or_dis {
        d20 = match adv_or_dis.as_str() {
            "adv" | "advantage" => "2d20K1",
            "dis" | "disadvantage" => "2d20k1",
            _ => "2d20kl1",
        };
    }

    // While we're at it, convert the stat to a bonus
    let ability_mod = (((ability_score as f32) - 10f32) / 2f32).floor() as i32;
    let formula = format!("{} + {}", d20, proficiency + ability_mod);
    let result = Roller::new(&formula)?.roll()?;
    ctx.say(format!("Rolling {}: {} → {}", stat, formula, result)).await?;

    Ok(())
}

async fn get_world(client: &FoundryClient) -> Result<DND5EWorld, Box<dyn std::error::Error + Send + Sync>> {
    let payload = client.emit("world", Payload::Text(vec![])).await.unwrap();
    let json = to_one_json(payload);
    let raw_world = json.get(0).unwrap();
    let raw_world_debug = serde_json::to_string_pretty(json.get(0).unwrap()).unwrap();
    let mut file = File::create("foo.json").await?;
    file.write_all(raw_world_debug.as_bytes()).await?;
    let deser = &mut serde_json::Deserializer::from_str(&raw_world_debug);
    let debug_world: Result<DND5EWorld, _> = serde_path_to_error::deserialize(deser);
    match debug_world {
        Ok(_) => (),
        Err(err) => {
            let path = err.path().to_string();
            println!("PATH: {}\nERR: {}\n", path, err.inner());
        }
    }
    let world: DND5EWorld = serde_json::from_value(raw_world.clone())?;
    Ok(world)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up foundry client
    let host = Args::parse().host;
    let foundry = FoundryClient::new(&host, "Voyeur", "").await?;

    // Set up discord client
    let token = env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    // Set up persistence
    let store = PickleDb::load("janusdb", PickleDbDumpPolicy::AutoDump, SerializationMethod::Json)
        .unwrap_or_else(|_| PickleDb::new("janusdb", PickleDbDumpPolicy::AutoDump, SerializationMethod::Json));

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![roll(), assoc()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(DiscordState {
                    foundry,
                    store: tokio::sync::Mutex::new(store)
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();

    println!("Finished");
    // socket.disconnect().expect("Disconnect failed");
    Ok(())
}
