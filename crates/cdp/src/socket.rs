use futures_util::StreamExt;
use fxhash::FxHashMap;
use std::net::TcpListener;
use std::process::Output;
use std::sync::Arc;
use tachyonix::{self, Sender};
use tokio::sync::Mutex;
use tokio::{process::Command, sync::oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::Capabilities;

pub struct ChromiumBrowser {
    message_tx: Sender<Message>,
    // ws_sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    request_map: Arc<Mutex<FxHashMap<String, oneshot::Sender<Message>>>>,
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
        let request_map = Arc::new(Mutex::new(FxHashMap::default()));
        let request_map_f = request_map.clone();

        // todo: cap
        let (tx, mut rx) = tachyonix::channel(3);
        let (ws_stream, ..) = connect_async(ws_address).await.unwrap();
        let (write, read) = ws_stream.split();

        tokio::spawn(rx.map(Ok).forward(write));

        let asdasd = read.for_each(|message| async move {
            // let foo = request_map_f.clone();
            // let g = request_map_1.lock().await;
            let data = message.unwrap().into_data();
            // let g = request_map_1.lock().await;
            // g.get(k)
            // tokio::io::stdout().write_all(&data).await.unwrap();
        });
        tokio::spawn(asdasd);

        Self {
            message_tx: tx,
            request_map: request_map,
        }
    }

    // todo: args
    async fn launch_chromium(path: String, args: Vec<String>) -> String {
        // let remote_debugging_port = allocate_port().to_string().as_str();
        // let mut args = args.clone();
        // args.push(&(String::from("--remote-debugging-port=") + remote_debugging_port));
        let command = Command::new(path).args(args).output().await.unwrap();
        let Output { stdout, .. } = command;
        // println!("{:?}", command.stdout);
        // remote_debugging_port
        String::from("abcd")
    }
}

/// Ask the kernel to allocate a port from `ip_local_port_range`, then drop it
fn allocate_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}
