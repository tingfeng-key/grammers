use super::Client;
use grammers_mtsender::InvocationError;
use grammers_tl_types as tl;

impl Client {
    pub async fn get_full_user(
        &mut self,
        user_id: i64,
    ) -> Result<tl::enums::users::UserFull, InvocationError> {
        let input_user = tl::types::InputUser {
            user_id,
            access_hash: 0i64,
        };
        let user_full = self
            .invoke(&tl::functions::users::GetFullUser {
                id: tl::enums::InputUser::User(input_user),
            })
            .await?;

        Ok(user_full)
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
