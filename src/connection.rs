use crate::error::*;
use futures_util::FutureExt;
use rust_socketio::{
    asynchronous::{Client, ClientBuilder},
    Event, Payload, TransportType,
};
use tokio::sync::{broadcast};
use tokio::time::{sleep, timeout, Duration};
use url::Url;

async fn promise_socket_emit(socket: &Client, event: &str, payload: Payload, timeout: Duration) -> Option<Payload> {
    // Would prefer to use oneshot, but cannot - emit_with_ack takes a FnMut, not a FnOnce, and is therefore incompatible
    let (tx, mut rx) = broadcast::channel::<Payload>(1);

    // Send the message
    socket
        .emit_with_ack(
            event,
            payload,
            timeout,
            move |payload: Payload, _| {
                let listener_tx = tx.clone(); // This is the only way I could get this code to compile
                async move {
                    listener_tx.send(payload).expect("Cannot send payload");
                }
                .boxed()
            },
        )
        .await
        .expect("Server unreachable");

    // Await the value that the emit-with-ack has sent
    match rx.recv().await {
        Ok(p) => Some(p),
        Err(_) => None,
    }
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
pub struct FoundryClient {
    /// Our websocket
    pub socket: Client,
    /// Our non blocking http client, used for session acquisition & login
    pub http_client: reqwest::Client,
    /// The current session
    pub session_id: String,
    /// The user id associated with our current session
    pub user_id: String,
}

impl FoundryClientBuilder {
    async fn establish_session(mut self, host: &str) -> Result<Self, FoundryClientError> {
        // Establish a session
        let response = reqwest::get(format!("{}/join", host))
            .await
            .map_err(|err| FoundryClientError::JoinError(err))?;
        let session = response.cookies().find(|cookie| cookie.name() == "session");
        self.session_id = Some(
            session
                .expect("UNRECOVERABLE: Could not acquire session")
                .value()
                .to_string(),
        );
        Ok(self)
    }

    /// Build a new http client
    fn build_client(mut self) -> Self {
        // Create our http client
        self.http_client = Some(
            reqwest::ClientBuilder::new()
                .cookie_store(true)
                .build()
                .expect("UNRECOVERABLE: Unable to build HTTP client"),
        );
        self
    }

    async fn establish_socket(mut self, host: &str) -> Result<Self, FoundryClientError> {
        // Incorporate it into url
        let mut session_url = Url::parse(host).map_err(|err| FoundryClientError::URLError(err))?;
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
                .map_err(|err| FoundryClientError::SocketError(err))?,
        );
        Ok(self)
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

    pub async fn acquire_user_id(mut self) -> Result<Self, FoundryClientError> {
        if let Some(socket) = self.socket.as_ref() {
            let payload = promise_socket_emit(socket, "getJoinData", Payload::Text(vec![]), Duration::from_secs(2)).await;
            match payload {
                Some(Payload::Text(mut items)) => {
                    let item = items.pop().unwrap();
                    self.user_id = item[0]["users"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|user| user["name"] == "Voyeur")
                        .map(|x| x["_id"].to_string());
                }
                _ => panic!("UNRECOVERABLE: getJoinData returned non-json data"),
            };
        } else {
            panic!("UNRECOVERABLE: Must initialize socket before acquiring user id")
        }

        println!("Got userid: {:?}", self.user_id);
        if let None = self.user_id {
            return Err(FoundryClientError::NoUserError("Voyeur".into()));
        }

        Ok(self)
    }

    /// Finalize the values in the builder
    pub fn build(self) -> FoundryClient {
        FoundryClient {
            socket: self
                .socket
                .expect("Missing socket - be sure to establish_socket"),
            http_client: self
                .http_client
                .expect("Missing http client - be sure to build_client"),
            session_id: self
                .session_id
                .expect("Missing session id - be sure to establish_session"),
            user_id: self
                .user_id
                .expect("Missing user id - be sure to acquire_user_id"),
        }
    }
}

impl FoundryClient {
    pub async fn new(host: &str) -> Result<FoundryClient, Box<dyn std::error::Error>> {
        let client = FoundryClientBuilder::default()
            .build_client()
            .establish_session(host)
            .await?
            .establish_socket(host)
            .await?
            .wait_ready()
            .await
            .acquire_user_id()
            .await?
            .build();
        Ok(client)
    }

    pub async fn emit(&self, event: &str, payload: Payload) -> Option<Payload> {
        promise_socket_emit(&self.socket, event, payload, Duration::from_secs(5)).await
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
// Notes: game = Game.create used for in game view, and Setup.create used for auth, license, setup, etc
