use std::fmt;

use crate::types::Chat;

use super::Client;
use grammers_mtsender::InvocationError;
use grammers_session::PackedChat;
use grammers_tl_types as tl;

const MAX_PARTICIPANT_LIMIT: i32 = 200;

#[derive(Debug)]
pub enum ChannelError {
    NotFound,
    Other(InvocationError),
}
impl fmt::Display for ChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ChannelError::*;
        match self {
            NotFound => write!(f, "not found the channel"),
            Other(e) => write!(f, "channel error: {}", e),
        }
    }
}

impl std::error::Error for ChannelError {}

impl Client {
    pub async fn get_channels(
        &mut self,
        id: Vec<tl::enums::InputChannel>,
    ) -> Result<Vec<crate::types::Chat>, ChannelError> {
        match self
            .invoke(&tl::functions::channels::GetChannels { id })
            .await
        {
            Ok(tl::enums::messages::Chats::Chats(chats)) => {
                let mut res_chats = vec![];
                for chat in chats.chats {
                    res_chats.push(crate::types::chat::Chat::from_chat(chat))
                }
                Ok(res_chats)
            }
            Ok(tl::enums::messages::Chats::Slice(chat_slice)) => {
                let mut res_chats = vec![];
                for chat in chat_slice.chats {
                    res_chats.push(crate::types::chat::Chat::from_chat(chat))
                }
                Ok(res_chats)
            }
            Err(e) => Err(ChannelError::Other(e)),
        }
    }

    //get full channel
    pub async fn get_full_channel(
        &mut self,
        channel: tl::enums::InputChannel,
    ) -> Result<(tl::types::ChannelFull, tl::types::Channel), ChannelError> {
        match self
            .invoke(&tl::functions::channels::GetFullChannel { channel })
            .await
        {
            Ok(tl::enums::messages::ChatFull::Full(chat_full)) => {
                let full = match chat_full.full_chat {
                    tl::enums::ChatFull::ChannelFull(channel_full) => Ok(channel_full),
                    _ => Err(ChannelError::NotFound),
                }?;
                let base = match chat_full.chats.first() {
                    Some(tl::enums::Chat::Channel(channel)) => Ok(channel.clone()),
                    _ => Err(ChannelError::NotFound),
                }?;
                Ok((full, base))
            }
            Err(e) => Err(ChannelError::Other(e)),
        }
    }

    pub fn input_channel_for_access_hash(
        self,
        channel_id: i64,
        access_hash: i64,
    ) -> tl::enums::InputChannel {
        tl::types::InputChannel {
            channel_id,
            access_hash,
        }
        .into()
    }

    pub fn input_channel_for_message(
        self,
        peer: tl::enums::InputPeer,
        msg_id: i32,
        channel_id: i64,
    ) -> tl::enums::InputChannel {
        tl::types::InputChannelFromMessage {
            peer,
            msg_id,
            channel_id,
        }
        .into()
    }

    // get chat' members
    pub async fn get_chat_members<C: Into<PackedChat>>(
        &self,
        chat: C,
        filter: tl::enums::ChannelParticipantsFilter,
    ) -> Result<Vec<Chat>, InvocationError> {
        let chat = chat.into();
        let input_channel = tl::types::InputChannel {
            channel_id: chat.id,
            access_hash: chat.access_hash.unwrap_or(0i64),
        };

        let mut request = tl::functions::channels::GetParticipants {
            channel: tl::enums::InputChannel::Channel(input_channel),
            filter,
            offset: 0,
            limit: MAX_PARTICIPANT_LIMIT,
            hash: 0,
        };
        let mut chat_members: Vec<Chat> = vec![];
        loop {
            if let tl::enums::channels::ChannelParticipants::Participants(p) =
                self.invoke(&request).await?
            {
                for elem in p.users {
                    chat_members.push(Chat::from_user(elem));
                }
                if request.offset >= p.count {
                    break;
                }

                if request.limit >= p.count {
                    break;
                }

                if (request.offset + request.limit) >= p.count {
                    break;
                }
                request.offset += request.limit;
            }
        }
        Ok(chat_members)
    }
}
