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

    // Number of times to greet
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
}

fn main() {
    let args = Args::parse();

    // define a callback which is called when a payload is received
    // this callback gets the payload as well as an instance of the
    // socket to communicate with the server
    let callback = |payload: Payload, socket: RawClient| {
        #[allow(deprecated)] match payload {
            Payload::Text(values) => println!("Received: {:#?}", values),
            Payload::Binary(bin_data) => println!("Received bytes: {:#?}", bin_data),
            // This payload type is deprecated, use Payload::Text instead
            Payload::String(str) => println!("Received: {}", str),
        }
        socket.emit("test", json!({"got ack": true})).expect("Server unreachable")
    };

    // get a socket that is connected to the admin namespace
    let mut url = Url::parse(&args.host).expect("Invalid URL");
    url.path_segments_mut().unwrap().push("socket.io/");
    url.query_pairs_mut().append_pair("session", "70ac46975b48d030bae1f821");
    ClientBuilder::new(url)
        .transport_type(TransportType::Websocket)
        .reconnect(true)
        .reconnect_on_disconnect(true)
        .reconnect_delay(500, 500)
        .max_reconnect_attempts(10)
        .on("test", callback)
        .on("error", |err, _| eprintln!("Received Error: {:#?}", err))
        .connect()
        .expect("Connection error");



/*

        // .expect("Connection failed");

    // emit to the "foo" event
    let json_payload = json!({"token": 123});
    socket.emit("foo", json_payload).expect("Server unreachable");

    // define a callback, that's executed when the ack got acked
    let ack_callback = |message: Payload, _| {
        println!("Yehaa! My ack got acked?");
        println!("Ack data: {:#?}", message);
    };

    let json_payload = json!({"myAckData": 123});
    // emit with an ack
    socket
        .emit_with_ack("test", json_payload, Duration::from_secs(2), ack_callback)
        .expect("Server unreachable");

    socket.disconnect().expect("Disconnect failed")

 */
}

// Notes: game = Game.create used for in game view, and Setup.create used for auth, license, setup, etc