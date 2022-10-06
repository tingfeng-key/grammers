use super::Client;
use grammers_mtsender::InvocationError;
use grammers_tl_types as tl;

impl Client {
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
