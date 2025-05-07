use clap::Parser;
use futures_util::FutureExt;
use rust_socketio::{
    asynchronous::{Client, ClientBuilder},
    Event, Payload, TransportType,
};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{sleep, Duration};
use url::Url;

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

#[derive(Default)]
struct FoundryClientBuilder {
    /// Our websocket
    socket: Option<Client>,
    /// Our blocking http client, used for session acquisition & login
    http_client: Option<reqwest::Client>,
    /// The current session
    session_id: Option<String>,
    /// The user id associated with our current session
    user_id: Option<String>,
}

/// Essentially the fully built version of the above
struct FoundryClient {
    /// Our websocket
    socket: Client,
    /// Our blocking http client, used for session acquisition & login
    http_client: reqwest::Client,
    /// The current session
    session_id: String,
    /// The user id associated with our current session
    user_id: String,
}

impl FoundryClientBuilder {
    async fn establish_session(mut self) -> Self {
        let args = Args::parse();

        // Establish a session
        let response = reqwest::get(format!("{}/join", args.host))
            .await
            .expect("Could not connect to join page. Is the world started?");
        let session = response.cookies().find(|cookie| cookie.name() == "session");
        self.session_id = Some(session.expect("Could not find session").value().to_string());
        self
    }

    /// Build a new http client
    fn build_client(mut self) -> Self {
        // Create our http client
        self.http_client = Some(
            reqwest::ClientBuilder::new()
                .cookie_store(true)
                .build()
                .expect("Error building reqwest::Client"),
        );
        self
    }

    async fn establish_socket(mut self) -> Self {
        let args = Args::parse();

        // Incorporate it into url
        let mut session_url = Url::parse(&args.host).expect("Invalid URL");
        session_url.path_segments_mut().unwrap().push("socket.io/"); // Technically should be out of here, but this flag is stupid anyway
        session_url
            .query_pairs_mut()
            .append_pair("session", &self.session_id.clone().unwrap());

        // Establish a socket
        let generic_callback = |evt: Event, payload: Payload, _: Client| {
            async move {
                println!("Unhandled event: {:?}", evt);
                print_payload(payload);
            }
            .boxed()
        };
        self.socket = Some(
            ClientBuilder::new(session_url)
                .transport_type(TransportType::Websocket)
                .reconnect(true)
                .reconnect_on_disconnect(true)
                .reconnect_delay(500, 500)
                .max_reconnect_attempts(10)
                .on_any(generic_callback)
                .connect()
                .await
                .expect("Connection error"),
        );
        self
    }

    /// Wait until socket and other components are ready
    async fn wait_ready(self) -> Self {
        // TODO: Do this better
        sleep(Duration::from_secs(3)).await;
        self
    }

    /*
    fn make_closure() -> impl FnMut() -> u32 {
        let mut i = 0;
        move || {
            i += 1;
            i
        }
    }
    */

    pub async fn acquire_user_id(mut self) -> Self {
        if let Some(socket) = self.socket.as_ref() {
            // Would prefer to use oneshot, but cannot - emit_with_ack takes a FnMut, not a FnOnce, and is therefore incompatible
            let (tx, mut rx) = broadcast::channel::<String>(1);

            // Send the message
            socket
                .emit_with_ack(
                    "getJoinData",
                    Payload::Text(vec![]),
                    Duration::from_secs(2),
                    move |payload: Payload, _| {
                        let listener_tx = tx.clone(); // This is the only way I could get this code to compile
                        async move {
                            print_payload(payload.clone());
                            match payload {
                                Payload::Text(mut items) => {
                                    let item = items.pop().unwrap();
                                    for user in item[0]["users"].as_array().unwrap() {
                                        if user["name"] == "Voyeur" {
                                            listener_tx
                                                .send(user["id"].to_string())
                                                .expect("Failed to send recovered user id");
                                            return;
                                        }
                                    }
                                    panic!("Unable to find a user named 'Voyeur'");
                                }
                                _ => {}
                            }
                        }
                        .boxed()
                    },
                )
                .await
                .expect("Server unreachable");

            // Await the value that the emit-with-ack has sent
            let user_id = rx.recv().await.expect("Something went wrong while waiting for 'getJoinData' to return. Perhaps it did not find a user id, or perhaps the connection failed");
            println!("Acquire user ID: {}", user_id);
            self.user_id = Some(user_id);
        } else {
            panic!("Must initialize socket before acquiring user id")
        }

        self
    }

    /// Finalize the values in the builder
    pub fn build(self) -> FoundryClient {
        FoundryClient {
            socket: self.socket.expect("Missing socket"),
            http_client: self.http_client.expect("Missing http client"),
            session_id: self.session_id.expect("Missing session id"),
            user_id: self.user_id.expect("Missing user id"),
        }
    }
}

/*
fn login_as_user(user_id: &str, socket: RawClient) {
    let args = Args::parse();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    let response = client.post(format!("{}/join", args.host))
        .headers(headers)
        .body(json!({
            "action": "join",
            "password": "",
            "userid": user_id
        }).to_string())
        .send().expect("Login failed");
    println!("{:#?}", response);
}
*/

fn print_payload(payload: Payload) {
    #[allow(deprecated)]
    match payload {
        Payload::Text(values) => println!("{:#?}", values),
        Payload::Binary(bin_data) => println!("{:#?}", bin_data),
        Payload::String(str) => println!("{:#?}", str),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _client = FoundryClientBuilder::default()
        .build_client()
        .establish_session()
        .await
        .establish_socket()
        .await
        .wait_ready()
        .await
        .acquire_user_id()
        .await
        .build();
    sleep(Duration::from_secs(10)).await;
    // socket.disconnect().expect("Disconnect failed");
    Ok(())
}

// Notes: game = Game.create used for in game view, and Setup.create used for auth, license, setup, etc
