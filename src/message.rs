//! Message types for websocket communication

use crate::error::WsError;

/// Messages that can be sent or received through the websocket
#[derive(Debug, Clone, PartialEq)]
pub enum WsMessage {
    /// Text message
    Text(String),
    /// Binary message
    Binary(Vec<u8>),
}

impl WsMessage {
    /// Create a text message
    pub fn text<S: Into<String>>(s: S) -> Self {
        WsMessage::Text(s.into())
    }

    /// Create a binary message
    pub fn binary<B: Into<Vec<u8>>>(b: B) -> Self {
        WsMessage::Binary(b.into())
    }

    /// Check if this is a text message
    pub fn is_text(&self) -> bool {
        matches!(self, WsMessage::Text(_))
    }

    /// Check if this is a binary message
    pub fn is_binary(&self) -> bool {
        matches!(self, WsMessage::Binary(_))
    }

    /// Get the text content if this is a text message
    pub fn as_text(&self) -> Option<&str> {
        match self {
            WsMessage::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get the binary content if this is a binary message
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            WsMessage::Binary(b) => Some(b),
            _ => None,
        }
    }

    /// Convert this message into a text string
    pub fn into_text(self) -> Result<String, WsError> {
        match self {
            WsMessage::Text(s) => Ok(s),
            _ => Err(WsError::InvalidMessage("Not a text message".to_string())),
        }
    }

    /// Convert this message into binary data
    pub fn into_binary(self) -> Result<Vec<u8>, WsError> {
        match self {
            WsMessage::Binary(b) => Ok(b),
            _ => Err(WsError::InvalidMessage("Not a binary message".to_string())),
        }
    }
}

impl From<String> for WsMessage {
    fn from(s: String) -> Self {
        WsMessage::Text(s)
    }
}

impl From<&str> for WsMessage {
    fn from(s: &str) -> Self {
        WsMessage::Text(s.to_string())
    }
}

impl From<Vec<u8>> for WsMessage {
    fn from(b: Vec<u8>) -> Self {
        WsMessage::Binary(b)
    }
}

impl From<&[u8]> for WsMessage {
    fn from(b: &[u8]) -> Self {
        WsMessage::Binary(b.to_vec())
    }
}
