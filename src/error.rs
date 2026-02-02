//! Error types for websocket operations

use thiserror::Error;

/// Result type alias for websocket operations
pub type WsResult<T> = Result<T, WsError>;

/// Errors that can occur during websocket operations
#[derive(Error, Debug, Clone)]
pub enum WsError {
    /// Failed to create or connect to the websocket
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Error sending a message through the websocket
    #[error("Send error: {0}")]
    SendError(String),

    /// Error receiving a message from the websocket
    #[error("Receive error: {0}")]
    ReceiveError(String),

    /// The websocket connection was closed
    #[error("Connection closed: code={code}, reason={reason}")]
    ConnectionClosed { code: u16, reason: String },

    /// The websocket encountered an error event
    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    /// Channel communication error
    #[error("Channel error: {0}")]
    ChannelError(String),

    /// Invalid message format
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

impl From<tokio::sync::mpsc::error::SendError<crate::message::WsMessage>> for WsError {
    fn from(e: tokio::sync::mpsc::error::SendError<crate::message::WsMessage>) -> Self {
        WsError::ChannelError(format!("Failed to send message to channel: {}", e))
    }
}
