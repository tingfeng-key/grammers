use std::fmt;

use super::Client;
use grammers_mtsender::InvocationError;
use grammers_tl_types as tl;

#[derive(Debug)]
pub enum ChatError {
    NotFound,
    JoinError,
    Other(InvocationError),
}
impl fmt::Display for ChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ChatError::*;
        match self {
            NotFound => write!(f, "not found the chat"),
            JoinError => write!(f, "join chat error"),
            Other(e) => write!(f, "chat error: {}", e),
        }
    }
}

impl std::error::Error for ChatError {}

impl Client {
    pub async fn get_chats(&mut self, id: Vec<i64>) -> Result<Vec<crate::types::Chat>, ChatError> {
        match self.invoke(&tl::functions::messages::GetChats { id }).await {
            Ok(tl::enums::messages::Chats::Chats(chats)) => Ok(chats
                .chats
                .into_iter()
                .map(|item| crate::types::Chat::from_chat(item))
                .collect()),
            Ok(tl::enums::messages::Chats::Slice(chat_slice)) => Ok(chat_slice
                .chats
                .into_iter()
                .map(|item| crate::types::Chat::from_chat(item))
                .collect()),
            Err(e) => Err(ChatError::Other(e)),
        }
    }

    pub async fn join_chat(&mut self, chat_id: &str) -> Result<crate::types::Chat, ChatError> {
        use tl::enums::Updates;
        const INVITE_LINK: &str = "https://t.me/joinchat/";
        let updates = match chat_id.starts_with(INVITE_LINK) {
            true => {
                let hash = chat_id.replace(INVITE_LINK, "");
                self.invoke(&tl::functions::messages::ImportChatInvite { hash })
                    .await
            }
            false => {
                let chat = self
                    .resolve_username(chat_id)
                    .await
                    .map_err(|e| ChatError::Other(e))?;

                if chat.is_none() {
                    return Err(ChatError::JoinError);
                }
                let chat = chat.unwrap();
                self.invoke(&tl::functions::channels::JoinChannel {
                    channel: tl::types::InputChannel {
                        channel_id: chat.id(),
                        access_hash: chat.access_hash().unwrap_or_default(),
                    }
                    .into(),
                })
                .await
            }
        }
        .map_err(|e| ChatError::Other(e))?;

        match updates {
            Updates::Combined(updates) => match updates.chats.first() {
                Some(chat) => Ok(crate::types::Chat::from_chat(chat.clone())),
                None => Err(ChatError::JoinError),
            },
            Updates::Updates(updates) => match updates.chats.first() {
                Some(chat) => Ok(crate::types::Chat::from_chat(chat.clone())),
                None => Err(ChatError::JoinError),
            },
            _ => Err(ChatError::JoinError),
        }
    }
}
