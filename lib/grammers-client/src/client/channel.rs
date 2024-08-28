use crate::types::Chat;

use super::Client;
use grammers_mtsender::{InvocationError, RpcError};
use grammers_session::PackedChat;
use grammers_tl_types as tl;

const MAX_PARTICIPANT_LIMIT: i32 = 200;

impl Client {
    pub async fn get_channels(
        &self,
        id: Vec<tl::enums::InputChannel>,
    ) -> Result<Vec<crate::types::Chat>, InvocationError> {
        match self
            .invoke(&tl::functions::channels::GetChannels { id })
            .await
        {
            Ok(tl::enums::messages::Chats::Chats(chats)) => {
                let mut res_chats = vec![];
                for chat in chats.chats {
                    res_chats.push(crate::types::chat::Chat::from_raw(chat))
                }
                Ok(res_chats)
            }
            Ok(tl::enums::messages::Chats::Slice(chat_slice)) => {
                let mut res_chats = vec![];
                for chat in chat_slice.chats {
                    res_chats.push(crate::types::chat::Chat::from_raw(chat))
                }
                Ok(res_chats)
            }
            Err(e) => Err(e),
        }
    }

    //get full channel
    pub async fn get_full_channel(
        &self,
        channel: tl::enums::InputChannel,
    ) -> Result<(tl::types::ChannelFull, crate::types::Chat), InvocationError> {
        match self
            .invoke(&tl::functions::channels::GetFullChannel { channel })
            .await
        {
            Ok(tl::enums::messages::ChatFull::Full(chat_full)) => {
                let full = match chat_full.full_chat {
                    tl::enums::ChatFull::ChannelFull(channel_full) => Ok(channel_full),
                    _ => Err(InvocationError::Rpc(RpcError {
                        code: 404,
                        name: "not found channel".to_string(),
                        value: None,
                        caused_by: None,
                    })),
                }?;
                let base = match chat_full.chats.first() {
                    Some(tl::enums::Chat::Channel(channel)) => {
                        Ok(crate::types::Chat::from_raw(channel.clone().into()))
                    }
                    _ => Err(InvocationError::Rpc(RpcError {
                        code: 404,
                        name: "not found channel".to_string(),
                        value: None,
                        caused_by: None,
                    })),
                }?;
                Ok((full, base))
            }
            Err(e) => Err(e),
        }
    }

    pub fn input_channel_for_access_hash(
        &self,
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
        &self,
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

    pub async fn get_chat_member<C: Into<PackedChat>>(
        &self,
        chat: C,
        filter: tl::enums::ChannelParticipantsFilter,
        page: i32,
    ) -> Result<(Vec<Chat>, i32), InvocationError> {
        use tl::enums::channels::ChannelParticipants::Participants;
        use tl::functions::channels::GetParticipants;

        let chat = chat.into();
        let input_channel = tl::types::InputChannel {
            channel_id: chat.id,
            access_hash: chat.access_hash.unwrap_or(0i64),
        };

        let request = GetParticipants {
            channel: tl::enums::InputChannel::Channel(input_channel),
            filter,
            offset: (MAX_PARTICIPANT_LIMIT * page),
            limit: MAX_PARTICIPANT_LIMIT,
            hash: 0,
        };

        if let Participants(p) = self.invoke(&request).await? {
            let chat_map = crate::types::ChatMap::new(p.users.clone(), p.chats);
            return Ok((
                p.users
                    .iter()
                    .map(|x| {
                        crate::utils::always_find_entity(
                            &crate::types::User::from_raw(x.clone()).pack().to_peer(),
                            &chat_map,
                            self,
                        )
                    })
                    .collect::<Vec<Chat>>(),
                p.count,
            ));
        }
        Ok((vec![], 0))
    }
}
