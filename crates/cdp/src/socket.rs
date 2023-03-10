use futures_util::StreamExt;
use fxhash::FxHashMap;
use serde_json::Value;
use std::net::TcpListener;
use std::process::{Output, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tachyonix::{self, Sender};
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::Mutex;
use tokio::{process::Command, sync::oneshot};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Message},
};

use crate::Capabilities;

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

pub struct RequestStorage {
    request_id: Arc<AtomicU32>,
    request_map: Arc<Mutex<FxHashMap<u32, oneshot::Sender<Value>>>>,
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

pub struct ChromiumBrowser {
    // request_id: u32,
    message_tx: Sender<Message>,
    request_storage: RequestStorage,
    // ws_sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    // request_map: Arc<Mutex<FxHashMap<String, oneshot::Sender<Message>>>>,
}

impl ChromiumBrowser {
    pub async fn new(capabilities: Capabilities) -> Self {
        let ws_address = if capabilities.launch {
            Self::launch_chromium(capabilities.binary, capabilities.args).await
        } else if !capabilities.launch && capabilities.debugger_address.is_empty() {
            panic!("Capabilities.debugger_address has to be set if launch is disabled")
        } else {
            capabilities.debugger_address
        };

        // should we use a sync mutex for less overhead since this is not expected to block?
        // let request_map = Arc::new(Mutex::new(FxHashMap::default()));

        let request_storage = RequestStorage::new();

        // todo: cap
        let (tx, rx) = tachyonix::channel(3);
        let (ws_stream, ..) = connect_async(ws_address).await.unwrap();
        let (write, mut read) = ws_stream.split();

        tokio::spawn(rx.map(Ok).forward(write));

        {
            let request_map = request_storage.request_map.clone();
            tokio::spawn(async move {
                // println!()
                let mut request_map = request_map.lock().await;
                while let Some(res) = read.next().await {
                    match res {
                        // todo: acutal error handling and maybe get_mut to drop key?
                        Ok(msg) => {
                            let msg = msg.to_text().unwrap();
                            let msg: Value = serde_json::from_str(msg).unwrap();
                            // Ok: Text("{\"id\":0,\"error\":{\"code\":-32601,\"message\":\"'Page.navigate' wasn't found\"}}")
                            // println!("{:?}", msg);
                            let rq_id = msg["id"].as_u64().unwrap() as u32;
                            // remove to take ownership
                            let sender = request_map.remove(&rq_id).unwrap();
                            sender.send(msg).unwrap();

                            // println!("Ok: {:?}", x)
                        }
                        Err(e) => println!("{:?}", e),
                    }
                }
            });
        }

        // let asdasd = read.for_each(|message| async move {
        //     // let store = Arc::clone(&request_map_f);
        //     // let foo = request_map_f.clone();
        //     // let g = request_map_1.lock().await;
        //     let data = message.unwrap().into_data();
        //     // let g = request_map_1.lock().await;
        //     // g.get(k)
        //     // tokio::io::stdout().write_all(&data).await.unwrap();
        // });
        // tokio::spawn(asdasd);

        Self {
            request_storage,
            // request_id: 0,
            message_tx: tx,
            // request_map: request_map,
        }
    }

    // todo: no Box<dyn Error>
    pub async fn run_command(
        &self,
        command: &mut Value,
    ) -> Result<Value, oneshot::error::RecvError> {
        let next_request_id = self.request_storage.next_request_id();
        command["id"] = next_request_id.into();
        let (tx, rx) = oneshot::channel();
        println!("oneshot::channel()");
        let mut rs = self.request_storage.request_map.lock().await;
        println!("request_map.lock()");
        // onesh
        rs.insert(next_request_id, tx);
        drop(rs);

        println!("rs.insert()");

        // let r = self.next_request_id();

        self.message_tx
            .send(Message::Text(command.to_string()))
            .await
            .unwrap();

        println!("self.message_tx.send()");

        rx.await
    }

    // todo: args
    async fn launch_chromium(path: String, args: Vec<String>) -> String {
        // todo: maybe we should save the handle and use .kill_on_drop(true) ?
        let mut command = Command::new(path)
            .args(args)
            // .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // .stdin(Stdio::piped())
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
            // println!("line: {}", line);
            if line.starts_with("DevTools listening on ") {
                debugging_address = line[22..].to_string();
                break;
            }
        }
        println!("{}", debugging_address);
        if debugging_address.is_empty() {
            panic!("debugging_address was not set, can't continue");
        }
        debugging_address.to_string()
    }
}

/// Ask the kernel to allocate a port from `ip_local_port_range`, then drop it
fn allocate_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}
