use super::Client;
use grammers_mtsender::InvocationError;
use grammers_tl_types as tl;

impl Client {
    pub async fn get_full_user(
        &mut self,
        id: tl::enums::InputUser,
    ) -> Result<tl::enums::users::UserFull, InvocationError> {
        let user_full = self
            .invoke(&tl::functions::users::GetFullUser { id })
            .await?;

        Ok(user_full)
    }

    pub async fn get_users(
        &mut self,
        id: Vec<tl::enums::InputUser>,
    ) -> Result<Vec<crate::types::chat::Chat>, InvocationError> {
        let users = self.invoke(&tl::functions::users::GetUsers { id }).await?;
        println!("t_len:{}", users.len());
        let mut chats = vec![];
        for user in users {
            chats.push(crate::types::chat::Chat::from_user(user))
        }
        Ok(chats)
    }

    pub fn input_user_for_access_hash(
        self,
        user_id: i64,
        access_hash: i64,
    ) -> tl::enums::InputUser {
        tl::types::InputUser {
            user_id,
            access_hash,
        }
        .into()
    }

    pub fn input_user_for_message(
        self,
        peer: tl::enums::InputPeer,
        user_id: i64,
        msg_id: i32,
    ) -> tl::enums::InputUser {
        tl::types::InputUserFromMessage {
            peer,
            msg_id,
            user_id,
        }
        .into()
    }

    // pub async fn get_full(&mut self, packed_chat: PackedChat) -> Result<Chat, InvocationError> {
    //     Ok(match packed_chat.ty {
    //         PackedType::User | PackedType::Bot => {
    //             let mut res = self
    //                 .invoke(&tl::functions::users::GetFullUser {
    //                     id: tl::enums::InputUser::User(packed_chat.to_input_peer()),
    //                 })
    //                 .await?;
    //             tl::enums::ChatFull::Full()
    //             Chat::from_user(res.pop().unwrap())
    //         }
    //         PackedType::Chat => {
    //             let mut res = match self
    //                 .invoke(&tl::functions::messages::GetChats {
    //                     id: vec![packed_chat.id],
    //                 })
    //                 .await?
    //             {
    //                 tl::enums::messages::Chats::Chats(chats) => chats.chats,
    //                 tl::enums::messages::Chats::Slice(chat_slice) => chat_slice.chats,
    //             };
    //             if res.len() != 1 {
    //                 panic!("fetching only one chat should exactly return one chat");
    //             }
    //             Chat::from_chat(res.pop().unwrap())
    //         }
    //         PackedType::Megagroup | PackedType::Broadcast | PackedType::Gigagroup => {
    //             let mut res = match self
    //                 .invoke(&tl::functions::channels::GetChannels {
    //                     id: vec![tl::enums::InputChannel::Channel(tl::types::InputChannel {
    //                         channel_id: packed_chat.id,
    //                         access_hash: packed_chat.access_hash.unwrap(),
    //                     })],
    //                 })
    //                 .await?
    //             {
    //                 tl::enums::messages::Chats::Chats(chats) => chats.chats,
    //                 tl::enums::messages::Chats::Slice(chat_slice) => chat_slice.chats,
    //             };
    //             if res.len() != 1 {
    //                 panic!("fetching only one chat should exactly return one chat");
    //             }
    //             Chat::from_chat(res.pop().unwrap())
    //         }
    //     })
    // }
}
