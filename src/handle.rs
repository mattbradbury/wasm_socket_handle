//! WsHandle - A handle to control websockets that implements Sink and Stream

use crate::error::{WsError, WsResult};
use crate::manager::{WsCommand, WsManager};
use crate::message::WsMessage;
use futures::{Sink, Stream};
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

pin_project! {
    /// A handle to a websocket that can be used in async contexts.
    ///
    /// `WsHandle` implements both `Sink` and `Stream` traits, allowing you to:
    /// - Send messages via `Sink::send()` or `SinkExt::send()`
    /// - Receive messages via `Stream::poll_next()` or `StreamExt::next()`
    ///
    /// The handle communicates with the actual websocket through channels.
    /// If you need to share the handle across multiple tasks, wrap it in an `Arc<Mutex<WsHandle>>`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wasm_socket_handle::{WsHandle, WsMessage};
    /// use futures::{SinkExt, StreamExt};
    ///
    /// async fn example() {
    ///     let mut ws = WsHandle::new("ws://example.com").unwrap();
    ///
    ///     // Send a message
    ///     ws.send(WsMessage::Text("Hello".to_string())).await.unwrap();
    ///
    ///     // Receive messages
    ///     if let Some(Ok(msg)) = ws.next().await {
    ///         println!("Received: {:?}", msg);
    ///     }
    /// }
    /// ```
    pub struct WsHandle {
        #[pin]
        rx_msg: mpsc::UnboundedReceiver<WsResult<WsMessage>>,
        tx_cmd: mpsc::UnboundedSender<WsCommand>,
        _manager: Option<WsManager>,
    }
}

impl WsHandle {
    /// Create a new websocket handle and connect to the given URL
    ///
    /// # Arguments
    ///
    /// * `url` - The websocket URL to connect to (e.g., "ws://example.com/socket")
    ///
    /// # Returns
    ///
    /// A new `WsHandle` instance or an error if the connection fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use wasm_socket_handle::WsHandle;
    ///
    /// let ws = WsHandle::new("ws://localhost:8080").unwrap();
    /// ```
    pub fn new(url: &str) -> WsResult<Self> {
        let (tx_msg, rx_msg) = mpsc::unbounded_channel();
        let (tx_cmd, rx_cmd) = mpsc::unbounded_channel();

        let manager = WsManager::new(url, tx_msg, rx_cmd)?;

        Ok(Self {
            rx_msg,
            tx_cmd,
            _manager: Some(manager),
        })
    }

    /// Manually close the websocket connection
    ///
    /// This sends a close command to the websocket. The connection will also
    /// be closed automatically when the handle is dropped.
    pub fn close(&self) -> WsResult<()> {
        self.tx_cmd
            .send(WsCommand::Close)
            .map_err(|e| WsError::ChannelError(format!("Failed to send close command: {}", e)))
    }

    /// Check if the handle is closed
    ///
    /// Returns true if the underlying websocket has been closed and no more
    /// messages can be sent or received.
    pub fn is_closed(&self) -> bool {
        self.tx_cmd.is_closed()
    }
}

// Implement Stream trait for receiving messages
impl Stream for WsHandle {
    type Item = WsResult<WsMessage>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        this.rx_msg.poll_recv(cx)
    }
}

// Implement Sink trait for sending messages
impl Sink<WsMessage> for WsHandle {
    type Error = WsError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Channel is always ready unless closed
        if self.is_closed() {
            Poll::Ready(Err(WsError::ConnectionClosed {
                code: 0,
                reason: "Connection closed".to_string(),
            }))
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: WsMessage) -> Result<(), Self::Error> {
        self.tx_cmd
            .send(WsCommand::Send(item))
            .map_err(|e| WsError::SendError(format!("Failed to send message: {}", e)))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Messages are sent immediately, no buffering
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let _ = self.close();
        Poll::Ready(Ok(()))
    }
}

// WsHandle is Send but not Sync because:
// - UnboundedReceiver<T> is Send but not Sync (single consumer)
// - UnboundedSender<T> is Send + Sync
// To share across tasks, wrap in Arc<Mutex<WsHandle>>

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_creation() {
        let text_msg = WsMessage::text("hello");
        assert!(text_msg.is_text());
        assert_eq!(text_msg.as_text(), Some("hello"));

        let binary_msg = WsMessage::binary(vec![1, 2, 3]);
        assert!(binary_msg.is_binary());
        assert_eq!(binary_msg.as_binary(), Some(&[1u8, 2, 3][..]));
    }

    #[test]
    fn test_ws_message_conversions() {
        let msg: WsMessage = "test".into();
        assert!(msg.is_text());

        let msg: WsMessage = vec![1, 2, 3].into();
        assert!(msg.is_binary());
    }
}
