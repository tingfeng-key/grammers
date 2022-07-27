use super::Client;
pub use grammers_mtsender::{AuthorizationError, InvocationError};
use grammers_session::PackedChat;
use grammers_tl_types as tl;

impl Client {
    pub async fn send_multi_media<
        C: Into<PackedChat>,
        M: Into<crate::types::InputSendMultiMedia>,
    >(
        &self,
        chat: C,
        input: M,
    ) -> Result<(), InvocationError> {
        let chat = chat.into();
        let input = input.into();
        println!("{:#?}", input.multi_media);
        let updates = self
            .invoke(&tl::functions::messages::SendMultiMedia {
                silent: input.silent,
                background: input.background,
                clear_draft: input.clear_draft,
                peer: chat.to_input_peer(),
                reply_to_msg_id: input.reply_to_msg_id,
                multi_media: input.multi_media,
                schedule_date: input.schedule_date,
                send_as: None,
            })
            .await?;

        println!("{:#?}", updates);
        Ok(())
    }
}
