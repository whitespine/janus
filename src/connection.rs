use crate::error::*;
use futures_util::FutureExt;
use rust_socketio::{
    asynchronous::{Client, ClientBuilder},
    Event, Payload, TransportType,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{mpsc};
use tokio::time::{sleep, Duration};
use url::Url;
use crate::error::FoundryClientError::FailedInit;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct JoinData {
    pub users: Vec<JoinDataUser>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct JoinDataUser {
    pub _id: String,
    pub name: String,
}

async fn promise_socket_emit(socket: &Client, event: &str, payload: Payload, timeout: Duration) -> Option<Payload> {
    // Would prefer to use oneshot, but cannot - emit_with_ack takes a FnMut, not a FnOnce, and is therefore incompatible
    let (tx, mut rx) = mpsc::channel::<Payload>(1);

    // Send the message
    // Timeout doesn't work. At least, it doesn't immediately
    socket
        .emit_with_ack(
            event,
            payload,
            timeout,
            move |payload: Payload, _| {
                let listener_tx = tx.clone(); // This is the only way I could get this code to compile
                async move {
                    listener_tx.send(payload).await.expect("Failed to send payload");
                }
                .boxed()
            },
        )
        .await
        .expect("Server unreachable");

    // Await the value that the emit-with-ack has sent
    let mut elapsed = Duration::from_secs(0);
    while rx.is_empty() && elapsed < timeout {
        sleep(Duration::from_millis(100)).await;
        elapsed += Duration::from_millis(100);
    }
    if rx.is_empty() {
        None
    } else {
        rx.recv().await
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
        if let Some(client) = self.http_client.as_mut() {
            let response = client.get(format!("{}/join", host))
                .send().await
                .map_err(|err| FoundryClientError::JoinError(err))?;
            let session = response.cookies().find(|cookie| cookie.name() == "session");
            self.session_id = Some(
                session
                    .expect("UNRECOVERABLE: Could not acquire session")
                    .value()
                    .to_string(),
            );
            Ok(self)
        } else {
            Err(FailedInit("http_client must be initialized first".into()))
        }
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
        let session_id = self.session_id.clone().ok_or(FailedInit("Need to acquire a session before establishing a socket".into()))?;
        let mut session_url = Url::parse(host).map_err(|err| FoundryClientError::URLError(err))?;
        session_url.path_segments_mut().unwrap().push("socket.io/"); // Technically should be out of here, but this flag is stupid anyway
        session_url
            .query_pairs_mut()
            .append_pair("session", &session_id);

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
                .opening_header("Cookie", format!("session={}", &session_id))
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
        sleep(Duration::from_secs(2)).await;
        Ok(self)
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

    pub async fn acquire_user_id(mut self, username: &str) -> Result<Self, FoundryClientError> {
        if let Some(socket) = self.socket.as_ref() {
            let payload = promise_socket_emit(socket, "getJoinData", Payload::Text(vec![]), Duration::from_secs(2)).await;
            match payload {
                Some(Payload::Text(items)) => {
                    let item = &items.get(0).unwrap();
                    let item = &item.as_array().unwrap()[0];
                    let data: JoinData = serde_json::from_value(item.clone()).expect("UNRECOVERABLE: getJoinData returned malformed data");
                    self.user_id = data.users.iter()
                        .find(|user| user.name == username)
                        .map(|x| x._id.clone());
                }
                _ => panic!("UNRECOVERABLE: getJoinData returned non-json data"),
            };
        } else {
            panic!("UNRECOVERABLE: Must initialize socket before acquiring user id")
        }

        println!("Got userid: {:?}", self.user_id);
        if let None = self.user_id {
            return Err(FoundryClientError::NoUserError(username.into()));
        }

        Ok(self)
    }

    pub async fn login(self, host: &str, password: &str) -> Result<Self, FoundryClientError> {
        if let Some(client) = self.http_client.as_ref() {
            let payload = json!({
                    "userid": self.user_id,
                    "password": password,
                    "action": "join"
                });
            // let response = client.post("https://echo.free.beeceptor.com")
            let response = client.post(format!("{}/join", host))
                .json(&payload)
                .send().await
                .map_err(|err| FoundryClientError::JoinError(err))?;
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
    pub async fn new(host: &str, username: &str, password: &str) -> Result<FoundryClient, Box<dyn std::error::Error>> {
        let client = FoundryClientBuilder::default()
            .build_client()
            .establish_session(host)
            .await?
            .establish_socket(host)
            .await?
            .acquire_user_id(username)
            .await?
            .login(host, password)
            .await?
            .establish_socket(host) // RE-establish, now with a logged in session
            .await?
            .build();
        Ok(client)
    }

    pub async fn emit(&self, event: &str, payload: Payload) -> Option<Payload> {
        promise_socket_emit(&self.socket, event, payload, Duration::from_secs(15)).await
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
