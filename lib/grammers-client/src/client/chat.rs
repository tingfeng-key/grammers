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
    pub async fn get_chats(&self, id: Vec<i64>) -> Result<Vec<Chat>, InvocationError> {
        match self.invoke(&tl::functions::messages::GetChats { id }).await {
            Ok(tl::enums::messages::Chats::Chats(chats)) => Ok(chats
                .chats
                .into_iter()
                .map(crate::types::Chat::from_raw)
                .collect()),
            Ok(tl::enums::messages::Chats::Slice(chat_slice)) => Ok(chat_slice
                .chats
                .into_iter()
                .map(crate::types::Chat::from_raw)
                .collect()),
            Err(e) => Err(e),
        }
    }

    /// Invite users to a channel/supergroup
    pub async fn invite_to_channel(
        &self,
        chat: PackedChat,
        users: Vec<PackedChat>,
    ) -> Result<Option<(Option<i64>, Vec<i64>)>, InvocationError> {
        use tl::enums::{messages::InvitedUsers, Updates};

        let user_ids = users
            .clone()
            .into_iter()
            .map(|x| x.id)
            .collect::<Vec<i64>>();

        let InvitedUsers::Users(invited_users) = self
            .invoke(&tl::functions::channels::InviteToChannel {
                channel: chat.to_input_channel_lossy(),
                users: users.into_iter().map(|x| x.to_input_user_lossy()).collect(),
            })
            .await?;

        let result = match invited_users.updates {
            Updates::Combined(update) => Some((
                update
                    .chats
                    .into_iter()
                    .map(|x| x.id())
                    .find(|x| x == &chat.id),
                update
                    .users
                    .into_iter()
                    .map(|x| x.id())
                    .filter(|x| user_ids.contains(x))
                    .collect::<Vec<i64>>(),
            )),
            Updates::Updates(update) => Some((
                update
                    .chats
                    .into_iter()
                    .map(|x| x.id())
                    .find(|x| x == &chat.id),
                update
                    .users
                    .into_iter()
                    .map(|x| x.id())
                    .filter(|x| user_ids.contains(x))
                    .collect::<Vec<i64>>(),
            )),
            _ => None,
        };
        Ok(result)
    }

    /// Adds a user to a chat and sends a service message on it
    pub async fn add_chat_user(
        &self,
        chat: PackedChat,
        user: PackedChat,
        fwd_limit: i32,
    ) -> Result<
        Option<(Option<crate::types::Chat>, Option<crate::types::chat::User>)>,
        InvocationError,
    > {
        use tl::enums::{messages::InvitedUsers, Updates};
        let InvitedUsers::Users(invited_users) = self
            .invoke(&tl::functions::messages::AddChatUser {
                chat_id: chat.id,
                user_id: user.to_input_user_lossy(),
                fwd_limit,
            })
            .await?;
        let result = match invited_users.updates {
            Updates::Combined(update) => Some((
                update
                    .chats
                    .into_iter()
                    .map(crate::types::chat::Chat::from_raw)
                    .find(|x| x.id() == chat.id),
                update
                    .users
                    .into_iter()
                    .map(crate::types::chat::User::from_raw)
                    .find(|x| x.id() == user.id),
            )),
            Updates::Updates(update) => Some((
                update
                    .chats
                    .into_iter()
                    .map(crate::types::chat::Chat::from_raw)
                    .find(|x| x.id() == chat.id),
                update
                    .users
                    .into_iter()
                    .map(crate::types::chat::User::from_raw)
                    .find(|x| x.id() == user.id),
            )),
            _ => None,
        };
        Ok(result)
    }
}
