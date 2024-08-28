use super::Client;
use grammers_mtsender::InvocationError;
use grammers_tl_types as tl;

impl Client {
    pub async fn contact_add(
        &self,
        add_phone_privacy_exception: bool,
        input_user: crate::types::extends::input_user::InputUser,
        first_name: String,
        last_name: String,
        phone: String,
    ) -> Result<Option<crate::types::chat::User>, InvocationError> {
        use tl::enums::InputUser;
        use tl::enums::Updates;

        let updates = self
            .invoke(&tl::functions::contacts::AddContact {
                add_phone_privacy_exception,
                id: input_user.0.clone(),
                first_name,
                last_name,
                phone,
            })
            .await?;
        let users_result = match updates {
            Updates::Combined(update) => Some(
                update
                    .users
                    .into_iter()
                    .map(crate::types::chat::User::from_raw)
                    .collect::<Vec<crate::types::chat::User>>(),
            ),
            Updates::Updates(update) => Some(
                update
                    .users
                    .into_iter()
                    .map(crate::types::chat::User::from_raw)
                    .collect::<Vec<crate::types::chat::User>>(),
            ),
            _ => None,
        };
        if let Some(users) = users_result {
            return match input_user.0 {
                InputUser::Empty => Ok(None),
                InputUser::UserSelf => Ok(users.into_iter().find(|x| x.is_self())),
                InputUser::User(input_user) => {
                    Ok(users.into_iter().find(|x| x.id() == input_user.user_id))
                }
                InputUser::FromMessage(msg) => {
                    Ok(users.into_iter().find(|x| x.id() == msg.user_id))
                }
            };
        }
        Ok(None)
    }

    pub async fn contact_search(
        &self,
        q: String,
        limit: i32,
    ) -> Result<Vec<crate::types::Chat>, InvocationError> {
        let mut chats = vec![];
        let tl::enums::contacts::Found::Found(found) = self
            .invoke(&tl::functions::contacts::Search { q, limit })
            .await?;

        for chat in found.chats.into_iter() {
            chats.push(crate::types::Chat::from_raw(chat));
        }
        for user in found.users.into_iter() {
            chats.push(crate::types::Chat::from_user(user));
        }
        Ok(chats)
    }

    pub async fn contacts_list(&self) -> Result<Vec<crate::types::Chat>, InvocationError> {
        use tl::enums::contacts::Contacts;
        Ok(
            match self
                .invoke(&tl::functions::contacts::GetContacts { hash: 0 })
                .await?
            {
                Contacts::Contacts(contacts) => contacts
                    .users
                    .into_iter()
                    .map(crate::types::Chat::from_user)
                    .collect::<Vec<crate::types::Chat>>(),
                Contacts::NotModified => vec![],
            },
        )
    }
}
