use std::fmt;

use super::Client;
use grammers_mtsender::InvocationError;
use grammers_session::PackedChat;
use grammers_tl_types as tl;

const MAX_PARTICIPANT_LIMIT: i32 = 200;

#[derive(Debug)]
pub enum ChannelError {
    NotFoundChannel,
    Other(InvocationError),
}
impl fmt::Display for ChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ChannelError::*;
        match self {
            NotFoundChannel => write!(f, "not found the channel"),
            Other(e) => write!(f, "channel error: {}", e),
        }
    }
}

impl std::error::Error for ChannelError {}

impl Client {
    //get full channel
    pub async fn get_full_channel(
        &mut self,
        chat: PackedChat,
    ) -> Result<tl::types::ChannelFull, ChannelError> {
        let input_channel = tl::types::InputChannel {
            channel_id: chat.id,
            access_hash: chat.access_hash.unwrap_or(0i64),
        };
        println!("{:#?}", input_channel);
        match self
            .invoke(&tl::functions::channels::GetFullChannel {
                channel: tl::enums::InputChannel::Channel(input_channel),
            })
            .await
        {
            Ok(tl::enums::messages::ChatFull::Full(chat_full)) => match chat_full.full_chat {
                tl::enums::ChatFull::ChannelFull(channel_full) => Ok(channel_full),
                _ => Err(ChannelError::NotFoundChannel),
            },
            Err(e) => Err(ChannelError::Other(e)),
        }
    }

    // get chat' members
    pub async fn get_chat_members<C: Into<PackedChat>>(
        &self,
        chat: C,
        filter: tl::enums::ChannelParticipantsFilter,
    ) -> Result<Vec<tl::enums::User>, InvocationError> {
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
        let mut chat_members: Vec<tl::enums::User> = vec![];
        loop {
            if let tl::enums::channels::ChannelParticipants::Participants(p) =
                self.invoke(&request).await?
            {
                for elem in p.users {
                    chat_members.push(elem);
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
