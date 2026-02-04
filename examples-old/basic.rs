//! Basic example demonstrating how to use WsHandle
//!
//! This example shows how to:
//! - Create a websocket connection
//! - Send messages
//! - Receive messages
//! - Handle errors

use futures::{SinkExt, StreamExt};
#[cfg(target_arch = "wasm32")]
use wasm_socket_handle::{WsHandle, WsMessage};

#[cfg(target_arch = "wasm32")]
#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logger
    wasm_logger::init(wasm_logger::Config::default());

    run_example().await;
}

#[cfg(not(target_arch = "wasm32"))]
#[main]
async fn main() {
    println!("This example is designed to run in a WASM environment.");
    println!("Please build with --target wasm32-unknown-unknown and run in a browser.");
}

#[cfg(target_arch = "wasm32")]
async fn run_example() {
    use web_sys::console;

    // Connect to a websocket server
    let ws_url = "ws://localhost:8080/socket";
    console::log_1(&format!("Connecting to {}...", ws_url).into());

    let mut ws = match WsHandle::new(ws_url).await {
        Ok(ws) => ws,
        Err(e) => {
            console::error_1(&format!("Failed to connect: {}", e).into());
            return;
        }
    };

    console::log_1(&"Connected!".into());

    // Send a greeting message
    if let Err(e) = ws.send(WsMessage::text("Hello from WASM!")).await {
        console::error_1(&format!("Failed to send: {}", e).into());
        return;
    }

    console::log_1(&"Message sent!".into());

    // Receive and process messages
    let mut message_count = 0;
    while let Some(result) = ws.next().await {
        match result {
            Ok(WsMessage::Text(text)) => {
                console::log_1(&format!("Received text: {}", text).into());
                message_count += 1;

                // Echo the message back
                let response = format!("Echo #{}: {}", message_count, text);
                if let Err(e) = ws.send(WsMessage::text(response)).await {
                    console::error_1(&format!("Failed to send echo: {}", e).into());
                }
            }
            Ok(WsMessage::Binary(data)) => {
                console::log_1(&format!("Received binary: {} bytes", data.len()).into());
                message_count += 1;

                // Echo binary data back
                if let Err(e) = ws.send(WsMessage::binary(data)).await {
                    console::error_1(&format!("Failed to send binary echo: {}", e).into());
                }
            }
            Err(e) => {
                console::error_1(&format!("Error receiving message: {}", e).into());
                break;
            }
        }

        // Stop after 10 messages for demo purposes
        if message_count >= 10 {
            console::log_1(&"Reached message limit, closing connection".into());
            break;
        }
    }

    // Close the connection
    if let Err(e) = ws.close() {
        console::error_1(&format!("Failed to close: {}", e).into());
    } else {
        console::log_1(&"Connection closed".into());
    }
}
