// use std::collections::HashMap;
// use fxhash::FxHashMap;

// use tokio::{net::TcpStream, sync::oneshot};
// use tokio_tungstenite::{
//     connect_async,
//     tungstenite::{connect, stream, Message, WebSocket},
//     MaybeTlsStream, WebSocketStream,
// };
// use crate::

// pub struct ChromiumBrowser {
//     websocket: WebSocket<MaybeTlsStream<TcpStream>>,
// }

// impl ChromiumBrowser {
//     pub fn connect_with_client(
//         client: &dyn ChromeAPI,
//         url: &String,
//     ) -> Result<ChromiumBrowser, Box<dyn Error>> {
//         let web_socket = ChromiumBrowser::get_websocket(client, &url)?;
//         trace!("Create websocket success");
//         Ok(ChromiumBrowser {
//             websocket: web_socket,
//         })
//     }

//     pub fn connect(url: &String) -> Result<ChromiumBrowser, Box<dyn Error>> {
//         let client = ChromeAPIClient {};

//         ChromiumBrowser::connect_with_client(&client, url)
//     }

//     fn get_websocket(
//         client: &dyn ChromeAPI,
//         url: &String,
//     ) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {
//         let response = client.get_websocket_session_url(&url)?;
//         let (ws_stream, _) = connect(&response[0].web_socket_debugger_url)?;
//         println!("WebSocket handshake has been successfully completed");

//         Ok(ws_stream)
//     }

//     pub fn run_command(&mut self, command: &mut Value) -> Result<(), Box<dyn Error>> {
//         command["id"] = 1.into();

//         self.websocket
//             .write_message(Message::Text(command.to_string()))?;

//         Ok(())
//     }
// }
// struct CDP {
//     rcp_tx: Sender<something>,
//     tx_map: HashMap<String, oneshot::Sender<String>>,
// }

// impl CDP {
//     fn new() -> Self {
//         CDP {

//             tx_map: HashMap::new(),
//         }
//     }

//     async fn rpc(&mut self, cmd: String) -> String {
//         let (tx, rx) = oneshot::channel();
//         let req_id = "12345";

//         self.tx_map.insert(req_id.to_string(), tx);
//         self.rcp_tx.send(Cmd { req_id, cmd })

//         rx.await.unwrap()
//     }
// }

pub use crate::caps::Capabilities;

pub mod caps;
pub mod socket;
pub mod cdp;

// pub async fn foo() {
//     let url = "ws://asdsa";

//     // let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
//     let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
//     println!("Hallo, Rust library here!")
// }
