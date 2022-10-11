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

    #[cfg(feature = "parse_invite_link")]
    fn parse_invite_link(invite_link: &str) -> Option<String> {
        let url_parse_result = url::Url::parse(invite_link);
        if url_parse_result.is_err() {
            return None;
        }

        let url_parse = url_parse_result.unwrap();
        let scheme = url_parse.scheme();
        let path = url_parse.path();
        if url_parse.host_str().is_none() || !vec!["https", "http"].contains(&scheme) {
            return None;
        }
        let host = url_parse.host_str().unwrap();
        let hosts = vec![
            "t.me",
            "telegram.me",
            "telegram.dog",
            "tg.dev",
            "telegram.me",
            "telesco.pe",
        ];

        if !hosts.contains(&host) {
            return None;
        }
        let paths = path.split("/").collect::<Vec<&str>>();

        if paths.len() == 1 {
            if paths[0].starts_with("+") {
                return Some(paths[0].replace("+", ""));
            }
            return None;
        }

        if paths.len() > 1 {
            if paths[0].starts_with("joinchat") {
                return Some(paths[1].to_string());
            }
            if paths[0].starts_with("+") {
                return Some(paths[0].replace("+", ""));
            }
            return None;
        }

        None
    }

    /// Accept an invite link to join the corresponding private chat.
    ///
    /// If the chat is public (has a public username), [`Client::join_chat`](Client::join_chat) should be used instead.
    pub async fn accept_invite_link(
        &mut self,
        invite_link: &str,
    ) -> Result<tl::enums::Updates, InvocationError> {
        #[cfg(not(feature = "parse_invite_link"))]
        let hash = invite_link.to_string();
        #[cfg(feature = "parse_invite_link")]
        let hash = {
            use grammers_mtproto::mtp::RpcError;
            let parse_result = Self::parse_invite_link(invite_link);
            if parse_result.is_none() {
                return Err(InvocationError::Rpc(RpcError {
                    code: 400,
                    name: "INVITE_HASH_INVALID".to_string(),
                    value: None,
                    caused_by: None,
                }));
            }
            parse_result.unwrap()
        };

        self.invoke(&tl::functions::messages::ImportChatInvite { hash })
            .await
    }

    /// Join a public group or channel.
    ///
    /// A channel is public if it has a username.
    /// To join private chats, [`Client::accept_invite_link`](Client::accept_invite_link) should be used instead.
    /// use PackedChat
    pub async fn join_chat<C: Into<PackedChat>>(
        &mut self,
        packed_chat: C,
    ) -> Result<Option<Chat>, InvocationError> {
        use tl::enums::Updates;

        let chat = packed_chat.into();
        let update_chat = match self
            .invoke(&tl::functions::channels::JoinChannel {
                channel: chat.try_to_input_channel().unwrap(),
            })
            .await?
        {
            Updates::Combined(updates) => Some(
                updates
                    .chats
                    .into_iter()
                    .filter(|x| x.id() == chat.id)
                    .collect::<Vec<tl::enums::Chat>>(),
            ),
            Updates::Updates(updates) => Some(
                updates
                    .chats
                    .into_iter()
                    .filter(|x| x.id() == chat.id)
                    .collect::<Vec<tl::enums::Chat>>(),
            ),
            _ => None,
        };

        match update_chat {
            Some(chats) if chats.len() > 0 => Ok(Some(Chat::from_chat(chats[0].clone()))),
            Some(chats) if chats.len() == 0 => Ok(None),
            None => Ok(None),
            Some(_) => Ok(None),
        }
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
