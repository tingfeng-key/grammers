use crate::types::Uploaded;
use grammers_tl_types as tl;
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEDULE_ONCE_ONLINE: i32 = 0x7ffffffe;

#[derive(Default)]
pub struct InputSendMultiMedia {
    pub(crate) silent: bool,
    pub(crate) background: bool,
    pub(crate) clear_draft: bool,
    pub(crate) reply_to_msg_id: Option<i32>,
    pub(crate) multi_media: Vec<tl::enums::InputSingleMedia>,
    pub(crate) schedule_date: Option<i32>,
}

impl InputSendMultiMedia {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn background(mut self, background: bool) -> Self {
        self.background = background;
        self
    }

    pub fn clear_draft(mut self, clear_draft: bool) -> Self {
        self.clear_draft = clear_draft;
        self
    }

    pub fn reply_to_msg_id(mut self, reply_to_msg_id: Option<i32>) -> Self {
        self.reply_to_msg_id = reply_to_msg_id;
        self
    }

    pub fn schedule_date(mut self, schedule_date: Option<SystemTime>) -> Self {
        self.schedule_date = schedule_date.map(|t| {
            t.duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i32)
                .unwrap_or(0)
        });
        self
    }

    pub fn schedule_once_online(mut self) -> Self {
        self.schedule_date = Some(SCHEDULE_ONCE_ONLINE);
        self
    }

    pub fn silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }
    fn sing_media(
        describe: impl Into<String>,
        media: tl::enums::InputMedia,
    ) -> tl::enums::InputSingleMedia {
        let random_id = crate::utils::generate_random_id();
        tl::types::InputSingleMedia {
            media,
            random_id,
            message: describe.into(),
            entities: None,
        }
        .into()
    }

    fn add_media(mut self, media: tl::enums::InputSingleMedia) -> Self {
        match self.multi_media.is_empty() {
            false => {
                self.multi_media.push(media);
            }
            true => {
                self.multi_media = vec![media];
            }
        };
        self
    }

    pub fn media<M: Into<tl::enums::MessageMedia>>(
        self,
        media: M,
        descript: Option<String>,
    ) -> Self {
        match media.into() {
            tl::enums::MessageMedia::Photo(media_photo) => {
                if let Some(tl::enums::Photo::Photo(photo)) = media_photo.photo {
                    return self.add_media(Self::sing_media(
                        descript.unwrap_or_default(),
                        tl::types::InputMediaPhoto {
                            id: tl::types::InputPhoto {
                                id: photo.id,
                                access_hash: photo.access_hash,
                                file_reference: photo.file_reference,
                            }
                            .into(),
                            ttl_seconds: None,
                            spoiler: true,
                        }
                        .into(),
                    ));
                }
                self
            }
            tl::enums::MessageMedia::Document(media_document) => {
                if let Some(tl::enums::Document::Document(document)) = media_document.document {
                    return self.add_media(Self::sing_media(
                        descript.unwrap_or_default(),
                        tl::types::InputMediaDocument {
                            id: tl::types::InputDocument {
                                id: document.id,
                                access_hash: document.access_hash,
                                file_reference: document.file_reference,
                            }
                            .into(),
                            ttl_seconds: None,
                            query: None,
                            spoiler: true,
                        }
                        .into(),
                    ));
                }
                self
            }
            _ => self,
        }
    }

    /// questions: https://stackoverflow.com/a/65881427
    // pub fn photo_url(self, url: impl Into<String>, descript: Option<String>) -> Self {
    //     let that = self.add_media(Self::sing_media(
    //         descript.unwrap_or_default(),
    //         tl::types::InputMediaPhotoExternal {
    //             url: url.into(),
    //             ttl_seconds: None,
    //         }
    //         .into(),
    //     ));
    //     that
    // }

    pub fn document(self, file: Uploaded, descript: Option<String>) -> Self {
        let mime_type = Self::get_file_mime(&file);
        let file_name = file.name().to_string();
        self.add_media(Self::sing_media(
            descript.unwrap_or_default(),
            tl::types::InputMediaUploadedDocument {
                nosound_video: false,
                force_file: false,
                file: file.raw,
                thumb: None,
                mime_type,
                attributes: vec![tl::types::DocumentAttributeFilename { file_name }.into()],
                stickers: None,
                ttl_seconds: None,
                spoiler: true,
            }
            .into(),
        ))
    }

    /// questions: https://stackoverflow.com/a/65881427
    // pub fn document_url(self, url: impl Into<String>, descript: Option<String>) -> Self {
    //     let that = self.add_media(Self::sing_media(
    //         descript.unwrap_or_default(),
    //         tl::types::InputMediaDocumentExternal {
    //             url: url.into(),
    //             ttl_seconds: None,
    //         }
    //         .into(),
    //     ));
    //     that
    // }

    pub fn get_file_mime(file: &Uploaded) -> String {
        if let Some(mime) = mime_guess::from_path(file.name()).first() {
            mime.essence_str().to_string()
        } else {
            "application/octet-stream".to_string()
        }
    }
}
