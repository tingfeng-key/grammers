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
    ) -> Result<Option<i32>, InvocationError> {
        let chat = chat.into();
        let input = input.into();

        let updates = self
            .invoke(&tl::functions::messages::SendMultiMedia {
                silent: input.silent,
                background: input.background,
                clear_draft: input.clear_draft,
                peer: chat.to_input_peer(),
                multi_media: input.multi_media,
                schedule_date: input.schedule_date,
                send_as: None,
                noforwards: false,
                update_stickersets_order: false,
                invert_media: false,
                reply_to: None,
            })
            .await?;

        Ok(match updates {
            tl::enums::Updates::UpdateShortSentMessage(update) => Some(update.id),
            tl::enums::Updates::UpdateShortMessage(update) => Some(update.id),
            tl::enums::Updates::UpdateShortChatMessage(update) => Some(update.id),
            _ => None,
        })
    }

    pub async fn upload_media(
        &self,
        chat: PackedChat,
        media: tl::enums::InputMedia,
    ) -> Result<tl::enums::MessageMedia, InvocationError> {
        self.invoke(&tl::functions::messages::UploadMedia {
            peer: chat.to_input_peer(),
            media,
        })
        .await
    }

    pub fn phone(&self, file: Uploaded) -> tl::enums::InputMedia {
        tl::types::InputMediaUploadedPhoto {
            file: file.input_file,
            stickers: None,
            ttl_seconds: None,
            spoiler: true,
        }
        .into()
    }

    pub fn video(&self, file: Uploaded) -> tl::enums::InputMedia {
        tl::types::InputMediaUploadedDocument {
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
                    duration: 0.0,
                    w: 0,
                    h: 0,
                    nosound: true,
                    preload_prefix_size: None,
                }
                .into(),
            ],
            stickers: None,
            ttl_seconds: None,
            spoiler: true,
        }
        .into()
    }

    pub fn document(&self, file: Uploaded) -> tl::enums::InputMedia {
        tl::types::InputMediaUploadedDocument {
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
            spoiler: true,
        }
        .into()
    }
}
