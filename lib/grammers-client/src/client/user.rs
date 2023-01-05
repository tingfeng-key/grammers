use crate::types::Chat;

use super::Client;
use grammers_mtsender::InvocationError;
use grammers_tl_types as tl;
use std::fmt;

#[derive(Debug)]
pub enum UserError {
    EmptyUser,
    NotSetPassword,
    Other(String),
}
impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UserError::*;
        match self {
            EmptyUser => write!(f, "empty user"),
            NotSetPassword => write!(f, "not set password"),
            Other(e) => write!(f, "{}", e),
        }
    }
}

impl From<InvocationError> for UserError {
    fn from(value: InvocationError) -> Self {
        Self::Other(value.to_string())
    }
}

impl Client {
    pub async fn get_full_user(
        &mut self,
        id: tl::enums::InputUser,
    ) -> Result<(tl::types::UserFull, Chat), UserError> {
        let tl::enums::users::UserFull::Full(user_full) = self
            .invoke(&tl::functions::users::GetFullUser { id })
            .await?;
        let full_user = match user_full.full_user {
            tl::enums::UserFull::Full(user) => user,
        };

        let user_base = match user_full.users.first() {
            Some(tl::enums::User::Empty(_)) => Err(UserError::EmptyUser),
            Some(tl::enums::User::User(user)) => Ok(Chat::from_user(user.clone().into())),
            None => Err(UserError::EmptyUser),
        }?;
        Ok((full_user, user_base))
    }

    pub async fn get_users(
        &mut self,
        id: Vec<tl::enums::InputUser>,
    ) -> Result<Vec<Chat>, InvocationError> {
        let users = self.invoke(&tl::functions::users::GetUsers { id }).await?;
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

    pub async fn update_profile(
        self,
        first_name: Option<String>,
        last_name: Option<String>,
        about: Option<String>,
    ) -> Result<crate::types::chat::User, InvocationError> {
        Ok(crate::types::chat::User::from_raw(
            self.invoke(&tl::functions::account::UpdateProfile {
                first_name,
                last_name,
                about,
            })
            .await?,
        ))
    }

    pub async fn enabled_password_verify(
        self,
        new_password: String,
        hint: Option<String>,
        email: Option<String>,
    ) -> Result<bool, UserError> {
        let password = self.get_password_information().await?;
        match !password.has_password() {
            true => {
                let (new_algo, new_hash) = password.generate_new_hash(new_password).unwrap();
                let request = tl::functions::account::UpdatePasswordSettings {
                    password: tl::enums::InputCheckPasswordSrp::InputCheckPasswordEmpty,
                    new_settings: tl::types::account::PasswordInputSettings {
                        new_algo: Some(new_algo.into()),
                        new_password_hash: Some(new_hash),
                        hint,
                        email,
                        new_secure_settings: None,
                    }
                    .into(),
                };
                println!("{:#?}", request);
                Ok(self.invoke(&request).await?)
            }
            false => Err(UserError::NotSetPassword),
        }
    }

    pub async fn change_password_verify(
        self,
        current_password: String,
        new_password: String,
        hint: Option<String>,
        email: Option<String>,
    ) -> Result<bool, UserError> {
        let password = self.get_password_information().await?;
        match password.has_password() {
            true => {
                let (new_algo, new_hash) = password.generate_new_hash(new_password).unwrap();
                let request = tl::functions::account::UpdatePasswordSettings {
                    password: password.to_input_check_password_srp(current_password),
                    new_settings: tl::types::account::PasswordInputSettings {
                        new_algo: Some(new_algo.into()),
                        new_password_hash: Some(new_hash),
                        hint,
                        email,
                        new_secure_settings: None,
                    }
                    .into(),
                };
                println!("{:#?}", request);
                Ok(self.invoke(&request).await?)
            }
            false => Err(UserError::NotSetPassword),
        }
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
