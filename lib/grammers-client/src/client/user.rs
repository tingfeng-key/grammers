use super::Client;
use crate::types::Chat;
use grammers_mtsender::{InvocationError, RpcError};
use grammers_tl_types as tl;

impl Client {
    pub async fn get_full_user(
        &self,
        id: tl::enums::InputUser,
    ) -> Result<(tl::types::UserFull, Chat), InvocationError> {
        let tl::enums::users::UserFull::Full(user_full) = self
            .invoke(&tl::functions::users::GetFullUser { id })
            .await?;
        let tl::enums::UserFull::Full(full_user) = user_full.full_user;

        let user_base = match user_full.users.first() {
            Some(tl::enums::User::Empty(_)) => Err(InvocationError::Rpc(RpcError {
                code: 404,
                name: "not found user".to_string(),
                value: None,
                caused_by: None,
            })),
            Some(tl::enums::User::User(user)) => Ok(Chat::from_user(user.clone().into())),
            None => Err(InvocationError::Rpc(RpcError {
                code: 404,
                name: "not found user".to_string(),
                value: None,
                caused_by: None,
            })),
        }?;
        Ok((full_user, user_base))
    }

    pub async fn get_users(
        &self,
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
        &self,
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
        &self,
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
        &self,
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

    pub async fn edit_2fa(
        &self,
        old_pwd: Option<String>,
        new_pwd: Option<String>,
        hint: Option<String>,
        email: Option<String>,
    ) -> Result<bool, EditTwoFaError> {
        if old_pwd.is_none() && new_pwd.is_none() {
            return Err(EditTwoFaError::Other(InvocationError::Rpc(RpcError {
                code: 400,
                name: "old password and new password cannot be empty".to_string(),
                value: None,
                caused_by: None,
            })));
        }
        let password = self
            .get_password_information()
            .await
            .map_err(EditTwoFaError::Other)?;
        let current_algo = password.algo(false);
        let mut new_algo: tl::enums::PasswordKdfAlgo = tl::types::PasswordKdfAlgoUnknown {}.into();
        let mut new_hash = vec![];
        if new_pwd.is_some() {
            let algo = password.algo(true);
            new_hash = password.generate_new_hash(algo.clone(), &new_pwd.unwrap_or_default());
            new_algo = algo.into();
        }

        let password = match !password.has_password() && old_pwd.is_none() {
            true => tl::enums::InputCheckPasswordSrp::InputCheckPasswordEmpty,
            false => {
                if old_pwd.is_none() {
                    return Err(EditTwoFaError::Other(InvocationError::Rpc(RpcError {
                        code: 400,
                        name: "old password cannot be empty".to_string(),
                        value: None,
                        caused_by: None,
                    })));
                }
                password
                    .to_input_check_password_srp(current_algo, &old_pwd.unwrap())
                    .await
            }
        };
        let request = tl::functions::account::UpdatePasswordSettings {
            password,
            new_settings: tl::types::account::PasswordInputSettings {
                new_algo: Some(new_algo),
                new_password_hash: Some(new_hash),
                hint: Some(hint.unwrap_or_default()),
                email,
                new_secure_settings: None,
            }
            .into(),
        };
        match self.invoke(&request).await {
            Ok(res) => Ok(res),
            Err(err) if err.is("EMAIL_UNCONFIRMED_*") => Err(EditTwoFaError::EmailUnconfirmed),
            Err(err) => Err(EditTwoFaError::Other(err)),
        }
    }

    pub async fn edit_2fa_email_code(&self, code: &str) -> Result<bool, EditTwoFaError> {
        self.invoke(&tl::functions::account::ConfirmPasswordEmail {
            code: code.to_owned(),
        })
        .await
        .map_err(EditTwoFaError::Other)
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum EditTwoFaError {
    EmailUnconfirmed,
    Other(InvocationError),
}

impl std::fmt::Display for EditTwoFaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmailUnconfirmed => write!(f, "please input email code",),
            Self::Other(e) => write!(f, "sign in error: {}", e),
        }
    }
}

impl std::error::Error for EditTwoFaError {}
