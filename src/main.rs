mod connection;
pub mod error;
mod world;
mod dnd5e;

use clap::Parser;
use rust_socketio::Payload;
use tokio::time::{sleep, Duration};
use crate::connection::FoundryClient;
use crate::dnd5e::DND5EWorld;

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
    let client = FoundryClient::new(&host).await?;

    let world = client.emit("world", Payload::Text(vec![])).await.unwrap();
    let world = to_one_json(world);
    let world: DND5EWorld = serde_json::from_value(world)?; // DND5EWorld::deserialize(to_one_json(world))?;

    // for(let )

    sleep(Duration::from_secs(10)).await;
    // socket.disconnect().expect("Disconnect failed");
    Ok(())
}

