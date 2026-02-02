# wasm_socket_handle

A Rust library using tokio to remotely control a web-sys websocket from a WsHandle that implements Sink and Stream.

## Overview

`wasm_socket_handle` provides a handle-based API for WebSocket communication in WebAssembly environments. The `WsHandle` struct implements both `Sink` and `Stream` traits from the `futures` crate, allowing seamless integration with async Rust code.

### Key Features

- **Sink/Stream Traits**: Use familiar `futures` traits for sending and receiving messages
- **Send + Sync**: WsHandle is safe to share across async tasks
- **Clone**: WsHandle can be cloned to use in multiple tasks simultaneously
- **Tokio Integration**: Built on tokio channels for efficient async communication
- **Type-Safe Messages**: Strong typing for text and binary WebSocket messages
- **Error Handling**: Comprehensive error types for all WebSocket operations
- **WASM-First**: Designed specifically for WebAssembly targets using web-sys
- **No Unsafe Code**: Pure safe Rust implementation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
wasm_socket_handle = "0.1"
```

## Usage

### Basic Example

```rust
use wasm_socket_handle::{WsHandle, WsMessage};
use futures::{SinkExt, StreamExt};

async fn example() {
    // Create a new websocket connection
    let mut ws = WsHandle::new("ws://example.com/socket").unwrap();
    
    // Send a text message
    ws.send(WsMessage::Text("Hello, server!".to_string())).await.unwrap();
    
    // Receive messages in a loop
    while let Some(result) = ws.next().await {
        match result {
            Ok(WsMessage::Text(text)) => println!("Received text: {}", text),
            Ok(WsMessage::Binary(data)) => println!("Received {} bytes", data.len()),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
```

### Sending Messages

The `WsHandle` implements `Sink<WsMessage>`, so you can use any sink methods:

```rust
use futures::SinkExt;

// Send and wait
ws.send(WsMessage::text("hello")).await?;

// Send multiple messages
ws.send(WsMessage::text("message 1")).await?;
ws.send(WsMessage::text("message 2")).await?;

// Send binary data
ws.send(WsMessage::binary(vec![1, 2, 3, 4])).await?;
```

### Receiving Messages

The `WsHandle` implements `Stream<Item = Result<WsMessage, WsError>>`:

```rust
use futures::StreamExt;

// Simple loop
while let Some(msg) = ws.next().await {
    match msg {
        Ok(WsMessage::Text(text)) => println!("Text: {}", text),
        Ok(WsMessage::Binary(data)) => println!("Binary: {} bytes", data.len()),
        Err(e) => println!("Error: {}", e),
    }
}
```

### Cloning and Sharing

WsHandle can be cloned to use in multiple tasks:

```rust
use futures::{SinkExt, StreamExt};

// Clone the handle for concurrent operations
let mut ws_receiver = ws.clone();
let mut ws_sender = ws.clone();

// Spawn a task for receiving
wasm_bindgen_futures::spawn_local(async move {
    while let Some(Ok(msg)) = ws_receiver.next().await {
        println!("Received: {:?}", msg);
    }
});

// Send from another task
ws_sender.send(WsMessage::text("hello")).await.unwrap();
```

### Message Types

Messages are represented by the `WsMessage` enum:

```rust
pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
}
```

Helper methods for creating messages:

```rust
// Create messages
let text_msg = WsMessage::text("hello");
let binary_msg = WsMessage::binary(vec![1, 2, 3]);

// Convert from types
let msg: WsMessage = "hello".into();
let msg: WsMessage = vec![1, 2, 3].into();
```

### Error Handling

All errors implement the `std::error::Error` trait:

```rust
use wasm_socket_handle::WsError;

match ws.send(msg).await {
    Ok(_) => println!("Message sent!"),
    Err(WsError::ConnectionClosed { code, reason }) => {
        println!("Connection closed: {} - {}", code, reason);
    }
    Err(e) => println!("Error: {}", e),
}
```

## Architecture

The library uses a channel-based architecture with safe concurrency:

1. **WsHandle**: The user-facing handle that implements Sink and Stream
   - Cloneable and shareable across tasks
   - Receiver wrapped in `Arc<Mutex<>>` for safe sharing
   - No unsafe code required
2. **WsManager**: Internal manager that interfaces with web-sys WebSocket
3. **Channels**: Tokio channels for communication between handle and manager
   - Command channel (unbounded): Handle → Manager for sending messages
   - Message channel (unbounded): Manager → Handle for receiving messages

This design allows the handle to be Send + Sync + Clone while the actual WebSocket operations happen in the WASM event loop.

## Requirements

- Rust 1.70 or later
- `wasm32-unknown-unknown` target for WebAssembly
- A WASM runtime environment (browser or similar)

## Building

For regular development (non-WASM):
```bash
cargo build
```

For WebAssembly target:
```bash
cargo build --target wasm32-unknown-unknown
```

## Testing

Run the test suite:
```bash
cargo test
```

For WASM-specific tests:
```bash
wasm-pack test --headless --firefox
```

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

