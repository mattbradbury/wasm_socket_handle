//! Advanced example showing multiple concurrent operations
//!
//! This example demonstrates:
//! - Using Arc<tokio::sync::Mutex<>> to share the handle across tasks
//! - Handling websocket in multiple async tasks
//! - Sending and receiving concurrently

use futures::{SinkExt, StreamExt};
use wasm_socket_handle::{WsHandle, WsMessage};

#[cfg(target_arch = "wasm32")]
#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    run_advanced_example().await;
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    println!("This example is designed to run in a WASM environment.");
}

#[cfg(target_arch = "wasm32")]
async fn run_advanced_example() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use web_sys::console;

    let ws_url = "ws://localhost:8080/socket";
    console::log_1(&format!("Connecting to {}...", ws_url).into());

    let ws = match WsHandle::new(ws_url) {
        Ok(ws) => Arc::new(Mutex::new(ws)),
        Err(e) => {
            console::error_1(&format!("Failed to connect: {}", e).into());
            return;
        }
    };

    console::log_1(&"Connected! Starting concurrent operations...".into());

    // Share the handle with the receiver task
    let ws_receiver = Arc::clone(&ws);

    // Spawn a task to handle incoming messages
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            let result = ws_receiver.lock().await.next().await;

            match result {
                Some(Ok(WsMessage::Text(text))) => {
                    console::log_1(&format!("[Receiver] Got text: {}", text).into());
                }
                Some(Ok(WsMessage::Binary(data))) => {
                    console::log_1(&format!("[Receiver] Got {} bytes", data.len()).into());
                }
                Some(Err(e)) => {
                    console::error_1(&format!("[Receiver] Error: {}", e).into());
                    break;
                }
                None => {
                    console::log_1(&"[Receiver] Stream ended".into());
                    break;
                }
            }
        }
    });

    // Share the handle with the sender task
    let ws_sender = Arc::clone(&ws);

    // Send a series of messages
    for i in 0..5 {
        let message = format!("Message number {}", i + 1);
        console::log_1(&format!("[Sender] Sending: {}", message).into());

        if let Err(e) = ws_sender.lock().await.send(WsMessage::text(message)).await {
            console::error_1(&format!("[Sender] Failed to send: {}", e).into());
            break;
        }

        // Wait 1 second (1000ms) between messages
        gloo_timers::future::TimeoutFuture::new(1000).await;
    }

    console::log_1(&"[Sender] Finished sending messages".into());

    // Wait 2 seconds (2000ms) for responses to come back
    gloo_timers::future::TimeoutFuture::new(2000).await;

    let ws = ws.lock().await;
    // Close the connection using the original handle
    if let Err(e) = ws.close() {
        console::error_1(&format!("Failed to close: {}", e).into());
    } else {
        console::log_1(&"Connection closed gracefully".into());
    }
}
