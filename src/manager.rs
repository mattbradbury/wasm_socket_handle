//! WebSocket manager that interfaces with web-sys WebSocket

#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
use web_sys::{BinaryType, CloseEvent, ErrorEvent, MessageEvent, WebSocket};

use crate::error::{WsError, WsResult};
use crate::message::WsMessage;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tracing;

/// Internal commands for controlling the websocket
#[derive(Debug)]
pub(crate) enum WsCommand {
    Send(WsMessage),
    Close,
}

/// Manager for the web-sys WebSocket, handles the actual websocket operations
#[cfg(target_arch = "wasm32")]
pub struct WsManager {
    ws: WebSocket,
    rx_cmd: mpsc::UnboundedReceiver<WsCommand>,
    tx_msg: mpsc::UnboundedSender<WsResult<WsMessage>>,
    _on_message_closure: Closure<dyn FnMut(MessageEvent)>,
    _on_error_closure: Closure<dyn FnMut(ErrorEvent)>,
    _on_close_closure: Closure<dyn FnMut(CloseEvent)>,
    // _on_open_closure: Closure<dyn FnMut(JsValue)>,
}

#[cfg(target_arch = "wasm32")]
impl WsManager {
    /// Create a new websocket manager and connect to the given URL
    pub(crate) async fn new(
        url: String,
        tx_msg: mpsc::UnboundedSender<WsResult<WsMessage>>,
        mut rx_cmd: mpsc::UnboundedReceiver<WsCommand>,
        is_ready: Arc<tokio::sync::watch::Sender<WsResult<()>>>,
    ) {
        // tracing::info!("Top of WsManager.  is_ready: {:?}", &is_ready);

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let _ = rt.enter();

        web_sys::console::log_1(&"WsManager::new(..)".into());
        tracing::info!("Initializing WebSocket manager for URL: {}", url);
        let ws = WebSocket::new(&url).map_err(|e| {
            tracing::error!("Failed to create WebSocket: {:?}", e);
            WsError::ConnectionError(format!("Failed to create WebSocket: {:?}", e))
        });
        let ws = match ws {
            Ok(ws) => ws,
            Err(e) => {
                _ = is_ready.clone().send(Err(e)).unwrap();
                return;
            }
        };

        tracing::debug!("WebSocket instance created");

        let (tx_open, mut rx_open) = watch::channel::<Option<Result<(), WsError>>>(None);

        // Set binary type to arraybuffer
        ws.set_binary_type(BinaryType::Arraybuffer);

        // Handle incoming messages
        let tx_msg_clone = tx_msg.clone();
        let on_message_closure = Closure::wrap(Box::new(move |e: MessageEvent| {
            tracing::debug!("Received WebSocket message event");
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let text = String::from(txt);
                tracing::trace!("Received text message: {}", text);
                let _ = tx_msg_clone.send(Ok(WsMessage::Text(text)));
            } else if let Ok(array_buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let array = js_sys::Uint8Array::new(&array_buffer);
                let data = array.to_vec();
                tracing::trace!("Received binary message of {} bytes", data.len());
                let _ = tx_msg_clone.send(Ok(WsMessage::Binary(data)));
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        ws.set_onmessage(Some(on_message_closure.as_ref().unchecked_ref()));

        // Handle errors
        let tx_msg_err = tx_msg.clone();
        let tx_open_err = tx_open.clone();
        let on_error_closure = Closure::wrap(Box::new(move |e: ErrorEvent| {
            tracing::error!("WebSocket error event: {:?}", e.message());
            let error = WsError::WebSocketError(format!("WebSocket error: {:?}", e.message()));
            let _ = tx_open_err.send(Some(Err(error.clone())));
            let _ = tx_msg_err.send(Err(error));
        }) as Box<dyn FnMut(ErrorEvent)>);

        ws.set_onerror(Some(on_error_closure.as_ref().unchecked_ref()));

        // Handle close events
        let tx_msg_close = tx_msg.clone();
        let tx_open_close = tx_open.clone();
        let on_close_closure = Closure::wrap(Box::new(move |e: CloseEvent| {
            tracing::info!("WebSocket closed: code={}, reason={}", e.code(), e.reason());
            let error = WsError::ConnectionClosed {
                code: e.code(),
                reason: e.reason(),
            };
            let _ = tx_open_close.send(Some(Err(error.clone())));
            let _ = tx_msg_close.send(Err(error));
        }) as Box<dyn FnMut(CloseEvent)>);

        ws.set_onclose(Some(on_close_closure.as_ref().unchecked_ref()));

        // Handle open event
        let tx_open_open = tx_open.clone();
        let on_open_closure = Closure::wrap(Box::new(move |_: JsValue| {
            tracing::info!("WebSocket connection opened");
            let _ = tx_open_open.send(Some(Ok(())));
            // Connection opened, ready to send/receive
        }) as Box<dyn FnMut(JsValue)>);

        ws.set_onopen(Some(on_open_closure.as_ref().unchecked_ref()));

        tracing::info!("is_ready {:?}", &is_ready);

        if let Err(e) = rx_open.changed().await.map_err(|_| {
            let res = rx_open.borrow();
            tracing::trace!("rx_open contents {:?}", dbg!(&res));

            WsError::ConnectionError("WebSocket open channel closed unexpectedly".to_string())
        }) {
            is_ready.send(Err(e)).ok();
            return;
        } else {
            let res = is_ready.send(Ok(()));
            tracing::trace!("{:?}", dbg!(&res));
            res.expect("Tried to say is_ready()");
        }

        // let mut this = Self {
        //     ws,
        //     _on_message_closure: on_message_closure,
        //     _on_error_closure: on_error_closure,
        //     _on_close_closure: on_close_closure,
        //     rx_cmd,
        //     tx_msg,
        // };

        // Spawn a task to handle outgoing commands
        // let ws_clone = ws.clone();
        // wasm_bindgen_futures::spawn_local(async move { this.run().await });
        //     this.run().await
        // }

        // pub async fn run(&mut self) {
        tracing::debug!("Entering while loop");
        while let Some(cmd) = rx_cmd.recv().await {
            tracing::debug!("Processing WebSocket command: {:?}", cmd);
            match cmd {
                WsCommand::Send(msg) => {
                    tracing::debug!("Sending message to WebSocket");
                    let result = match msg {
                        WsMessage::Text(text) => ws.send_with_str(&text),
                        WsMessage::Binary(data) => ws.send_with_u8_array(&data),
                    };

                    if let Err(e) = result {
                        tracing::error!("Failed to send message to WebSocket: {:?}", e);
                        // Could send error back through channel if needed
                        web_sys::console::error_1(&format!("Send error: {:?}", e).into());
                    }
                }
                WsCommand::Close => {
                    tracing::info!("Closing WebSocket connection");
                    let _ = ws.close();
                    break;
                }
            }
            tracing::debug!("At end of while loop");
        }
    }

    /// Close the websocket connection
    pub fn close(&self) -> WsResult<()> {
        tracing::info!("Manually closing WebSocket");
        self.ws
            .close()
            .map_err(|e| WsError::ConnectionError(format!("Failed to close: {:?}", e)))
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for WsManager {
    fn drop(&mut self) {
        // let _ = self.ws.close();
    }
}

// Non-WASM stub for compilation on other platforms
#[cfg(not(target_arch = "wasm32"))]
pub struct WsManager;

#[cfg(not(target_arch = "wasm32"))]
impl WsManager {
    pub fn new(
        _url: &str,
        _tx_msg: mpsc::UnboundedSender<WsResult<WsMessage>>,
        _rx_cmd: mpsc::UnboundedReceiver<WsCommand>,
    ) -> WsResult<Self> {
        Err(WsError::ConnectionError(
            "WebSocket is only supported on WASM target".to_string(),
        ))
    }

    pub fn close(&self) -> WsResult<()> {
        Ok(())
    }
}
