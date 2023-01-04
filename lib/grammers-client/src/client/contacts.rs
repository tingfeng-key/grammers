use super::Client;
use grammers_mtsender::InvocationError;
use grammers_tl_types as tl;

impl Client {
    pub async fn contact_add(
        self,
        add_phone_privacy_exception: bool,
        id: tl::enums::InputUser,
        first_name: String,
        last_name: String,
        phone: String,
    ) -> Result<Option<crate::types::chat::User>, InvocationError> {
        use tl::enums::InputUser;
        use tl::enums::Updates;

        let updates = self
            .invoke(&tl::functions::contacts::AddContact {
                add_phone_privacy_exception,
                id: id.clone(),
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
                    .map(|x| crate::types::chat::User::from_raw(x))
                    .collect::<Vec<crate::types::chat::User>>(),
            ),
            Updates::Updates(update) => Some(
                update
                    .users
                    .into_iter()
                    .map(|x| crate::types::chat::User::from_raw(x))
                    .collect::<Vec<crate::types::chat::User>>(),
            ),
            _ => None,
        };
        if let Some(users) = users_result {
            return match id {
                InputUser::Empty => Ok(None),
                InputUser::UserSelf => Ok(users.into_iter().filter(|x| x.is_self()).next()),
                InputUser::User(input_user) => Ok(users
                    .into_iter()
                    .filter(|x| x.id() == input_user.user_id)
                    .next()),
                InputUser::FromMessage(msg) => {
                    Ok(users.into_iter().filter(|x| x.id() == msg.user_id).next())
                }
            };
        }
        Ok(None)
    }

    pub async fn contact_search(
        self,
        q: String,
        limit: i32,
    ) -> Result<Vec<crate::types::Chat>, InvocationError> {
        let mut chats = vec![];
        let tl::enums::contacts::Found::Found(found) = self
            .invoke(&tl::functions::contacts::Search { q, limit })
            .await?;

        for chat in found.chats.into_iter() {
            chats.push(crate::types::Chat::from_chat(chat));
        }
        for user in found.users.into_iter() {
            chats.push(crate::types::Chat::from_user(user));
        }
        Ok(chats)
    }
}
