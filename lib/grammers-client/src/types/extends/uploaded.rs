use crate::types::Uploaded;
use grammers_tl_types as tl;

pub fn to_phone(uplaoded: Uploaded) -> tl::enums::InputMedia {
    tl::types::InputMediaUploadedPhoto {
        file: uplaoded.raw,
        stickers: None,
        ttl_seconds: None,
        spoiler: true,
    }
    .into()
}

pub fn to_video(uplaoded: &Uploaded) -> tl::enums::InputMedia {
    tl::types::InputMediaUploadedDocument {
        nosound_video: false,
        force_file: false,
        file: uplaoded.raw.clone(),
        thumb: None,
        mime_type: super::input_multi_media::InputSendMultiMedia::get_file_mime(uplaoded),
        attributes: vec![
            tl::types::DocumentAttributeFilename {
                file_name: uplaoded.name().to_string(),
            }
            .into(),
            tl::types::DocumentAttributeVideo {
                round_message: false,
                supports_streaming: true,
                duration: 0.0,
                w: 0,
                h: 0,
                nosound: false,
                preload_prefix_size: None,
                video_start_ts: None,
            }
            .into(),
        ],
        stickers: None,
        ttl_seconds: None,
        spoiler: true,
    }
    .into()
}
pub fn to_document(uplaoded: &Uploaded) -> tl::enums::InputMedia {
    tl::types::InputMediaUploadedDocument {
        nosound_video: false,
        force_file: false,
        file: uplaoded.raw.clone(),
        thumb: None,
        mime_type: super::input_multi_media::InputSendMultiMedia::get_file_mime(uplaoded),
        attributes: vec![tl::types::DocumentAttributeFilename {
            file_name: uplaoded.name().to_string(),
        }
        .into()],
        stickers: None,
        ttl_seconds: None,
        spoiler: true,
    }
    .into()
}
