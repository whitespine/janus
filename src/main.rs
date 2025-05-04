use std::thread::sleep;
use std::time::Duration;
use clap::Parser;
use rust_socketio::{TransportType};
use url::Url;
use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde_json::json;

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

fn on_ack(evt: &str, payload: Payload, _socket: RawClient) {
    println!("Ack {:?}", evt);
    print_payload(payload)
}

fn print_payload(payload: Payload) {
    #[allow(deprecated)] match payload {
        Payload::Text(values) => println!("{:#?}", values),
        Payload::Binary(bin_data) => println!("{:#?}", bin_data),
        Payload::String(str) => println!("{:#?}", str),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let args = Args::parse();

    let mut session = None;
    if args.add_session > 0 {
        let response = reqwest::blocking::get(format!("{}/join", args.host))?;
        session = Some(response.cookies().find(|cookie| cookie.name() == "session").expect("Failed to find session cookie").value().to_string());
    }

    // Build a url with session if specified
    let mut url = Url::parse(&args.host).expect("Invalid URL");
    if let Some(session) = session {
        url.path_segments_mut().unwrap().push("socket.io/"); // Technically should be out of here, but this flag is stupid anyway
        url.query_pairs_mut().append_pair("session", &session);
    }

    // Establish our client
    println!("Connecting to url {:?}", url);
    let socket = ClientBuilder::new(url)
        .transport_type(TransportType::Websocket)
        .reconnect(true)
        .reconnect_on_disconnect(true)
        .reconnect_delay(500, 500)
        .max_reconnect_attempts(10)
        .on_any(|evt, p, _| {
            println!("Event received {:#?}", evt);
            print_payload(p)
        })
        .connect()
        .expect("Connection error");


    // Say hello
    for _ in 0..2 {
        socket
            .emit_with_ack("getJoinData", Payload::Text(vec![]), Duration::from_secs(2), |p, s| on_ack("getJoinData", p, s))
            .expect("Server unreachable");
        sleep(Duration::from_secs(1));
    }

    sleep(Duration::from_secs(10));
    socket.disconnect().expect("Disconnect failed");
    Ok(())
}

// Notes: game = Game.create used for in game view, and Setup.create used for auth, license, setup, etc