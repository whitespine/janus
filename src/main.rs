mod connection;
pub mod error;
mod world;
mod dnd5e;

use clap::Parser;
use rust_socketio::Payload;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};
use crate::connection::FoundryClient;
use crate::dnd5e::{DND5EActor, DND5EWorld};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Url to connect to. Include http(s):// and any trailing suffix, if needed
    #[arg(long)]
    host: String,

    #[arg(long, action = clap::ArgAction::Count)]
    add_session: u8,
    // Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
}

fn to_one_json(payload: Payload) -> serde_json::Value {
    // if let Payload::Text(v) = payload;
    match payload {
        Payload::Text(mut items) => {
            items.remove(0)
        },
        _ => panic!("Not json!"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Make a basic client
    let host = Args::parse().host;
    let client = FoundryClient::new(&host, "Voyeur", "").await?;

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
    let world: DND5EWorld = serde_json::from_value(raw_world.clone())?; // DND5EWorld::deserialize(to_one_json(world))?;

    for actor in world.actors.iter() {
        match actor {
            DND5EActor::npc { base, .. } => {
                println!("Found an npc named {}", base.document.name)
            }
            DND5EActor::character { base, .. } => {
                println!("Found a player named {}", base.document.name)
            },
            _ => {}
        }
    }


    println!("Finished, waiting for shutdown");
    sleep(Duration::from_secs(10)).await;
    // socket.disconnect().expect("Disconnect failed");
    Ok(())
}