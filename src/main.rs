mod connection;
mod dnd5e;
pub mod error;
mod world;

use crate::connection::FoundryClient;
use crate::dnd5e::{DND5EActor, DND5EWorld};
use clap::Parser;
use rust_socketio::Payload;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use poise::serenity_prelude as serenity;
use std::env;
use kv::Integer;
use crate::error::CommandError;
use crate::error::CommandError::InvalidAttribute;

static PLAYERS_BUCKET: &str = "player_characters";

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
    store: kv::Store
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
    let user_id = kv::Integer::from(ctx.author().id.get());

    match world.actors.iter().find_map(|actor| {
        if let DND5EActor::character { base, .. } = actor {
            if base.document.name == name && base.document.id.is_some() {
                return Some(base.document.id.clone().unwrap());
            }
        }
        None
    }) {
        Some(id) => { // I clearly fucked up the typings here but... ???
            let bucket: kv::Bucket<Integer, String> = ctx.data().store.bucket(Some(PLAYERS_BUCKET))?;
            bucket.set(&user_id, &id)?;
            ctx.say(format!("Successfully associated with actor id {}", id)).await?;
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
) -> Result<(), DiscordError> {
    let foundry = &ctx.data().foundry;
    let world = get_world(foundry).await?;

    // Figure out who they should be
    let user_id = kv::Integer::from(ctx.author().id.get());
    let bucket: kv::Bucket<Integer, String> = ctx.data().store.bucket(Some(PLAYERS_BUCKET))?;
    let actor_id = bucket.get(&user_id)?;

    // See if it works
    let actor_id = actor_id.ok_or(CommandError::MissingAssocChar)?;

    // Attempt to find character system data
    let actor =  world.actors.iter().find(|actor| {
        if let DND5EActor::character { base, .. } = actor {
            return base.document.id == Some(actor_id.clone());
        }
        false
    }).ok_or(CommandError::InvalidAssocChar)?;
    let DND5EActor::character { system, .. } = actor else { Err(CommandError::InvalidAssocChar)? };

    // Do stuff conditionally
    let stat_value = match stat.as_str() {
        "str" | "strength" => system.abilities.str.value,
        "dex" | "dexterity" => system.abilities.str.value,
        "cha" | "charisma" => system.abilities.str.value,
        "int" | "intelligence" => system.abilities.str.value,
        "wis" | "wisdom" => system.abilities.str.value,
        "con" | "constitution" => system.abilities.str.value,
        _ => Err(InvalidAttribute(stat))?
    };
    ctx.say(format!("Found your character. They have a strength of {}", system.abilities.str.value)).await?;

    Ok(())
}

async fn get_world(client: &FoundryClient) -> Result<DND5EWorld, Box<dyn std::error::Error + Send + Sync>> {
    println!("Attempting get world");
    let payload = client.emit("world", Payload::Text(vec![])).await.unwrap();
    println!("Attempting parse world");
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
    let  cfg = kv::Config::new("./janus_db");
    let store = kv::Store::new(cfg)?;

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
                    store
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
