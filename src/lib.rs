//! # WASM Socket Handle
//!
//! A Rust library that provides a handle to control web-sys WebSockets using tokio streams.
//! The `WsHandle` implements both `Sink` and `Stream` traits, allowing you to send and receive
//! messages through channels while the actual websocket runs in the WASM environment.
//!
//! ## Features
//!
//! - **Send/Sync**: WsHandle can be safely shared across async tasks
//! - **Sink/Stream**: Standard futures traits for sending and receiving messages
//! - **Tokio integration**: Works seamlessly with tokio runtime
//! - **Error handling**: Comprehensive error types for websocket operations
//!
//! ## Example
//!
//! ```rust,no_run
//! use wasm_socket_handle::{WsHandle, WsMessage};
//! use futures::{SinkExt, StreamExt};
//!
//! async fn example() {
//!     let mut ws_handle = WsHandle::new("ws://example.com/socket").unwrap();
//!     
//!     // Send a message
//!     ws_handle.send(WsMessage::Text("Hello".to_string())).await.unwrap();
//!     
//!     // Receive messages
//!     while let Some(result) = ws_handle.next().await {
//!         match result {
//!             Ok(WsMessage::Text(text)) => println!("Received: {}", text),
//!             Ok(WsMessage::Binary(data)) => println!("Received {} bytes", data.len()),
//!             Err(e) => eprintln!("Error: {}", e),
//!         }
//!     }
//! }
//! ```

mod error;
mod handle;
mod manager;
mod message;

pub use error::{WsError, WsResult};
pub use handle::WsHandle;
pub use message::WsMessage;

#[cfg(target_arch = "wasm32")]
pub use manager::WsManager;
