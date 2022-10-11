use std::fmt;

use crate::types::Chat;

use super::Client;
use grammers_mtsender::InvocationError;
use grammers_session::PackedChat;
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
    pub async fn get_chats(&mut self, id: Vec<i64>) -> Result<Vec<Chat>, ChatError> {
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

    /// join private chat
    pub async fn accept_invite_link(
        &mut self,
        invite_link: &str,
    ) -> Result<Option<Chat>, InvocationError> {
        use tl::enums::Updates;
        assert!(invite_link.starts_with("https://t.me/joinchat/"));
        let update_chat = match self
            .invoke(&tl::functions::messages::ImportChatInvite {
                hash: invite_link.replace("https://t.me/joinchat/", ""),
            })
            .await?
        {
            Updates::Combined(updates) => updates.chats.first().cloned(),
            Updates::Updates(updates) => updates.chats.first().cloned(),
            _ => None,
        };

        if let Some(chat) = update_chat {
            return Ok(Some(Chat::from_chat(chat)));
        }
        Ok(None)
    }

    /// Join a group or channel.
    /// use PackedChat
    pub async fn join_chat(
        &mut self,
        packed_chat: PackedChat,
    ) -> Result<Option<Chat>, InvocationError> {
        use tl::enums::Updates;

        let update_chat = match self
            .invoke(&tl::functions::channels::JoinChannel {
                channel: packed_chat.to_input_channel_lossy(),
            })
            .await?
        {
            Updates::Combined(updates) => updates.chats.first().cloned(),
            Updates::Updates(updates) => updates.chats.first().cloned(),
            _ => None,
        };

        if let Some(chat) = update_chat {
            return Ok(Some(Chat::from_chat(chat)));
        }
        Ok(None)
    }

    /// Invite users to a channel/supergroup
    pub async fn invite_to_channel(
        &mut self,
        chat: PackedChat,
        users: Vec<PackedChat>,
    ) -> Result<bool, InvocationError> {
        let _ = self
            .invoke(&tl::functions::channels::InviteToChannel {
                channel: chat.to_input_channel_lossy(),
                users: users.into_iter().map(|x| x.to_input_user_lossy()).collect(),
            })
            .await?;
        Ok(true)
    }

    /// Adds a user to a chat and sends a service message on it
    pub async fn add_chat_user(
        &mut self,
        chat: PackedChat,
        user: PackedChat,
        fwd_limit: i32,
    ) -> Result<bool, InvocationError> {
        let _ = self
            .invoke(&tl::functions::messages::AddChatUser {
                chat_id: chat.id,
                user_id: user.to_input_user_lossy(),
                fwd_limit,
            })
            .await?;
        Ok(true)
    }
}
