use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use fxhash::FxHashMap;
use serde_json::{json, Value};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
// use std::net::TcpListener;
use std::process::Stdio;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tachyonix::{self, Sender};
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::Mutex;
use tokio::{process::Command, sync::oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::Capabilities;

// should we use a sync mutex for less overhead since this is not expected to block?
/// id -> oneshot::Sender
type RequestResolvers = Arc<Mutex<FxHashMap<u32, oneshot::Sender<Value>>>>;
/// sessionId -> Session
type SessionMap = Arc<Mutex<FxHashMap<String, Arc<Session>>>>;
/// targetId -> oneshot::Sender
type SessionsToAttach = Arc<Mutex<FxHashMap<String, oneshot::Sender<Arc<Session>>>>>;

#[derive(Debug)]
pub struct RequestStorage {
    request_id: Arc<AtomicU32>,
    request_map: RequestResolvers,
}
impl RequestStorage {
    fn new() -> Self {
        Self {
            request_id: Arc::new(AtomicU32::new(0)),
            request_map: Arc::new(Mutex::new(FxHashMap::default())),
        }
    }

    pub fn next_request_id(&self) -> u32 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }
}

#[derive(Debug)]
pub struct Session {
    // session_id: String,
    storage: RequestStorage,
}
impl Session {
    fn new() -> Self {
        Self {
            // session_id: session_id.to_string(),
            storage: RequestStorage::new(),
        }
    }
}

pub struct Connection {
    message_tx: Sender<Message>,
    browser_storage: RequestStorage,
    // sessions: SessionMap,
    sessions_to_attach: SessionsToAttach,
}

impl Connection {
    pub async fn new(capabilities: Capabilities) -> Self {
        let ws_address = if capabilities.launch {
            Self::launch_chromium(capabilities.binary, capabilities.args).await
        } else if !capabilities.launch && capabilities.debugger_address.is_empty() {
            panic!("Capabilities.debugger_address has to be set if launch is disabled")
        } else {
            capabilities.debugger_address
        };

        let browser_storage = RequestStorage::new();
        let sessions = Arc::new(Mutex::new(FxHashMap::default()));
        let sessions_to_attach = Arc::new(Mutex::new(FxHashMap::default()));

        // todo: set cap
        let (tx, rx) = tachyonix::channel(3);
        let (ws_stream, ..) = connect_async(ws_address).await.unwrap();
        let (write, read) = ws_stream.split();

        // messages -> ws
        tokio::spawn(rx.map(Ok).forward(write));
        // ws -> oneshot
        tokio::spawn(Self::handle_messages(
            read,
            browser_storage.request_map.clone(),
            sessions.clone(),
            sessions_to_attach.clone(),
        ));

        Self {
            sessions_to_attach,
            browser_storage,
            message_tx: tx,
            // sessions,
        }
    }

    async fn handle_messages(
        mut read_stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        request_map: RequestResolvers,
        sessions: SessionMap,
        sessions_to_attach: SessionsToAttach,
    ) {
        while let Some(res) = read_stream.next().await {
            match res {
                // todo: acutal error handling and maybe get_mut to drop key?
                Ok(msg) => {
                    let msg = msg.to_text().unwrap();
                    let msg: Value = serde_json::from_str(msg).unwrap();

                    log::debug!("<- {:?}", msg);

                    if let Some(method) = msg["method"].as_str() {
                        match method {
                            "Target.attachedToTarget" => {
                                log::debug!("Target.attachedToTarget");
                                let tid = msg["params"]["targetInfo"]["targetId"].as_str().unwrap();
                                let sid = msg["params"]["sessionId"].as_str().unwrap();
                                log::debug!("{}", sid);
                                // sessions_to_attach is a targetId -> oneshot mapping
                                let sender = sessions_to_attach.lock().await.remove(tid).unwrap();
                                let session = Arc::new(Session::new());

                                sessions
                                    .lock()
                                    .await
                                    .insert(sid.to_string(), session.clone());
                                sender.send(session).unwrap();
                            }
                            "Target.detachedFromTarget" => {
                                log::debug!("Target.detachedFromTarget");
                                todo!()
                            }
                            &_ => (),
                        };
                    }
                    if let Some(session_id) = msg["sessionId"].as_str() {
                        // todo: unnest and dry or sth
                        if let Some(id) = msg["id"].as_u64() {
                            let id = id as u32;
                            // let mut request_map = request_map.lock().await;

                            let session = sessions.lock().await;
                            let session = session
                                .get(session_id)
                                .expect("received an event for an unknown session");

                            let mut rm = session.storage.request_map.lock().await;

                            // remove to take ownership
                            let sender = rm.remove(&id).unwrap();
                            sender.send(msg).unwrap();
                        }
                        todo!()
                    } else if let Some(id) = msg["id"].as_u64() {
                        let id = id as u32;
                        let mut request_map = request_map.lock().await;
                        // remove to take ownership
                        let sender = request_map.remove(&id).unwrap();
                        sender.send(msg).unwrap();
                    }
                }
                Err(e) => log::error!("{:?}", e),
            }
        }
    }

    pub async fn run_browser_command(
        &self,
        command: &mut Value,
    ) -> Result<Value, oneshot::error::RecvError> {
        let next_request_id = self.browser_storage.next_request_id();
        command["id"] = next_request_id.into();

        let (tx, rx) = oneshot::channel();
        let mut rs = self.browser_storage.request_map.lock().await;
        rs.insert(next_request_id, tx);
        drop(rs);

        log::debug!("-> {:?}", command);

        self.message_tx
            .send(Message::Text(command.to_string()))
            .await
            .unwrap();

        rx.await
    }

    pub async fn run_session_command(
        &self,
        session: &Arc<Session>,
        command: &mut Value,
    ) -> Result<Value, oneshot::error::RecvError> {
        let next_request_id = session.storage.next_request_id();
        command["id"] = next_request_id.into();

        let (tx, rx) = oneshot::channel();
        let mut st = session.storage.request_map.lock().await;
        st.insert(next_request_id, tx);
        drop(st);

        log::debug!("-> {:?}", command);

        self.message_tx
            .send(Message::Text(command.to_string()))
            .await
            .unwrap();

        rx.await
    }

    pub async fn attach_to_target(
        &self,
        target_id: &str,
    ) -> Result<Arc<Session>, oneshot::error::RecvError> {
        let command = json!({
            "id": self.browser_storage.next_request_id(),
            "method": "Target.attachToTarget",
            "params": {
                "targetId": target_id,
                "flatten": true,
            }
        });
        let (tx, rx) = oneshot::channel();
        let mut sessions_to_attach = self.sessions_to_attach.lock().await;
        sessions_to_attach.insert(target_id.to_string(), tx);
        drop(sessions_to_attach);

        log::debug!("-> {:?}", command);

        self.message_tx
            .send(Message::Text(command.to_string()))
            .await
            .unwrap();

        rx.await
    }

    // todo: args
    async fn launch_chromium(path: String, args: Vec<String>) -> String {
        // todo: maybe we should save the handle and use .kill_on_drop(true) ?
        let mut command = Command::new(path)
            .args(args)
            .stderr(Stdio::piped())
            // .kill_on_drop(true)
            .spawn()
            .unwrap();

        let stderr = command.stderr.take().unwrap();
        let mut lines = io::BufReader::new(stderr).lines();
        let mut debugging_address = String::new();
        while let Some(line) = lines
            .next_line()
            .await
            .expect("no lines left, couldn't find debbuging_address")
        {
            if let Some(x) = line.strip_prefix("DevTools listening on ") {
                debugging_address = x.to_string();
                break;
            }
        }
        log::debug!("Debugging address: {}", debugging_address);
        if debugging_address.is_empty() {
            panic!("debugging_address was not set, can't continue");
        }
        debugging_address.to_string()
    }
}

// Ask the kernel to allocate a port from `ip_local_port_range`, then drop it
// fn allocate_port() -> u16 {
//     let listener = TcpListener::bind("127.0.0.1:0").unwrap();
//     let port = listener.local_addr().unwrap().port();
//     drop(listener);
//     port
// }
