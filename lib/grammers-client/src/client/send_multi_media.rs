use crate::types::{InputSendMultiMedia, Uploaded};

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

        // println!("{:#?}", input.multi_media);
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
                noforwards: false,
            })
            .await?;

        // println!("{:#?}", updates);
        Ok(())
    }

    pub async fn upload_media(
        &self,
        chat: PackedChat,
        media: tl::enums::InputMedia,
    ) -> Result<tl::enums::MessageMedia, InvocationError> {
        Ok(self
            .invoke(&tl::functions::messages::UploadMedia {
                peer: chat.to_input_peer(),
                media: media.into(),
            })
            .await?)
    }

    pub fn phone(&self, file: Uploaded) -> tl::enums::InputMedia {
        return tl::types::InputMediaUploadedPhoto {
            file: file.input_file,
            stickers: None,
            ttl_seconds: None,
        }
        .into();
    }

    pub fn video(&self, file: Uploaded) -> tl::enums::InputMedia {
        return tl::types::InputMediaUploadedDocument {
            nosound_video: false,
            force_file: false,
            file: file.clone().input_file,
            thumb: None,
            mime_type: InputSendMultiMedia::get_file_mime(&file),
            attributes: vec![
                tl::types::DocumentAttributeFilename {
                    file_name: file.name().to_string(),
                }
                .into(),
                tl::types::DocumentAttributeVideo {
                    round_message: false,
                    supports_streaming: true,
                    duration: 0,
                    w: 0,
                    h: 0,
                }
                .into(),
            ],
            stickers: None,
            ttl_seconds: None,
        }
        .into();
    }

    pub fn document(&self, file: Uploaded) -> tl::enums::InputMedia {
        return tl::types::InputMediaUploadedDocument {
            nosound_video: false,
            force_file: false,
            file: file.clone().input_file,
            thumb: None,
            mime_type: InputSendMultiMedia::get_file_mime(&file),
            attributes: vec![tl::types::DocumentAttributeFilename {
                file_name: file.name().to_string(),
            }
            .into()],
            stickers: None,
            ttl_seconds: None,
        }
        .into();
    }
}
