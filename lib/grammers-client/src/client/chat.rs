use std::fmt;

use super::Client;
use grammers_mtsender::InvocationError;
use grammers_tl_types as tl;

#[derive(Debug)]
pub enum ChatError {
    NotFound,
    Other(InvocationError),
}
impl fmt::Display for ChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ChatError::*;
        match self {
            NotFound => write!(f, "not found the chat"),
            Other(e) => write!(f, "chat error: {}", e),
        }
    }
}

impl std::error::Error for ChatError {}

impl Client {
    pub async fn get_full_chat(&mut self, chat_id: i64) -> Result<tl::types::ChatFull, ChatError> {
        match self
            .invoke(&tl::functions::messages::GetFullChat { chat_id })
            .await
        {
            Ok(tl::enums::messages::ChatFull::Full(chat_full)) => match chat_full.full_chat {
                tl::enums::ChatFull::Full(channel_full) => Ok(channel_full),
                _ => Err(ChatError::NotFound),
            },
            Err(e) => Err(ChatError::Other(e)),
        }
    }
}
