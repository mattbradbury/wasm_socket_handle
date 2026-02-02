//! Advanced example showing multiple concurrent operations
//!
//! This example demonstrates:
//! - Cloning the handle to use in multiple tasks
//! - Handling websocket in multiple async tasks
//! - Sending and receiving concurrently

use futures::{SinkExt, StreamExt};
use wasm_socket_handle::{WsHandle, WsMessage};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    
    run_advanced_example().await;
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("This example is designed to run in a WASM environment.");
}

#[cfg(target_arch = "wasm32")]
async fn run_advanced_example() {
    use web_sys::console;

    let ws_url = "ws://localhost:8080/socket";
    console::log_1(&format!("Connecting to {}...", ws_url).into());

    let ws = match WsHandle::new(ws_url) {
        Ok(ws) => ws,
        Err(e) => {
            console::error_1(&format!("Failed to connect: {}", e).into());
            return;
        }
    };

    console::log_1(&"Connected! Starting concurrent operations...".into());

    // Clone the handle for the receiver task
    let mut ws_receiver = ws.clone();
    
    // Spawn a task to handle incoming messages
    wasm_bindgen_futures::spawn_local(async move {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(WsMessage::Text(text)) => {
                    console::log_1(&format!("[Receiver] Got text: {}", text).into());
                }
                Ok(WsMessage::Binary(data)) => {
                    console::log_1(&format!("[Receiver] Got {} bytes", data.len()).into());
                }
                Err(e) => {
                    console::error_1(&format!("[Receiver] Error: {}", e).into());
                    break;
                }
            }
        }
        
        console::log_1(&"[Receiver] Stream ended".into());
    });

    // Clone the handle for the sender task
    let mut ws_sender = ws.clone();
    
    // Send a series of messages
    for i in 0..5 {
        let message = format!("Message number {}", i + 1);
        console::log_1(&format!("[Sender] Sending: {}", message).into());
        
        if let Err(e) = ws_sender.send(WsMessage::text(message)).await {
            console::error_1(&format!("[Sender] Failed to send: {}", e).into());
            break;
        }
        
        // Simulate some work between messages
        gloo_timers::future::TimeoutFuture::new(1000).await;
    }

    console::log_1(&"[Sender] Finished sending messages".into());
    
    // Give time for responses to come back
    gloo_timers::future::TimeoutFuture::new(2000).await;
    
    // Close the connection using the original handle
    if let Err(e) = ws.close() {
        console::error_1(&format!("Failed to close: {}", e).into());
    } else {
        console::log_1(&"Connection closed gracefully".into());
    }
}
