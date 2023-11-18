use crate::types;
use grammers_tl_types as tl;

#[derive(Debug)]
pub enum EntityType {
    Unknown,
    Mention,
    Hashtag,
    BotCommand,
    Url,
    Email,
    Bold,
    Italic,
    Code,
    Pre(String),
    TextUrl(String),
    MentionName(i64),
    InputMessageEntityMentionName(types::input_user::InputUser),
    Phone,
    Cashtag,
    Underline,
    Strike,
    BankCard,
    Spoiler,
    CustomEmoji(i64),
    Blockquote,
}

impl From<tl::enums::MessageEntity> for EntityType {
    fn from(entity: tl::enums::MessageEntity) -> Self {
        use tl::enums::MessageEntity as TlMessageEntity;
        match entity {
            TlMessageEntity::Unknown(_) => Self::Unknown,
            TlMessageEntity::Mention(_) => Self::Mention,
            TlMessageEntity::Hashtag(_) => Self::Hashtag,
            TlMessageEntity::BotCommand(_) => Self::BotCommand,
            TlMessageEntity::Url(_) => Self::Url,
            TlMessageEntity::Email(_) => Self::Email,
            TlMessageEntity::Bold(_) => Self::Bold,
            TlMessageEntity::Italic(_) => Self::Italic,
            TlMessageEntity::Code(_) => Self::Code,
            TlMessageEntity::Pre(pre) => Self::Pre(pre.language.clone()),
            TlMessageEntity::TextUrl(text_url) => Self::TextUrl(text_url.url.clone()),
            TlMessageEntity::MentionName(user) => Self::MentionName(user.user_id),
            TlMessageEntity::InputMessageEntityMentionName(user) => {
                Self::InputMessageEntityMentionName(types::input_user::InputUser::_from_raw(
                    user.user_id.clone(),
                ))
            }
            TlMessageEntity::Phone(_) => Self::Phone,
            TlMessageEntity::Cashtag(_) => Self::Cashtag,
            TlMessageEntity::Underline(_) => Self::Underline,
            TlMessageEntity::Strike(_) => Self::Strike,
            TlMessageEntity::BankCard(_) => Self::BankCard,
            TlMessageEntity::Spoiler(_) => Self::Spoiler,
            TlMessageEntity::CustomEmoji(emoji) => Self::CustomEmoji(emoji.document_id),
            TlMessageEntity::Blockquote(_) => Self::Blockquote,
        }
    }
}

#[derive(Debug)]
pub struct Entity {
    r#type: EntityType,
    text: String,
}

impl Entity {
    pub(crate) fn _from_message(
        text: &str,
        message_entities: Option<Vec<tl::enums::MessageEntity>>,
    ) -> Vec<Self> {
        let mut entities = vec![];
        if let Some(original_entities) = message_entities {
            for msg_entity in original_entities {
                entities.push(Self::_tranform(text, &msg_entity));
            }
        }
        entities
    }

    fn _tranform(message_text: &str, entity: &tl::enums::MessageEntity) -> Self {
        let text_u16 = message_text
            .encode_utf16()
            .skip(entity.offset() as usize)
            .take(entity.length() as usize)
            .collect::<Vec<u16>>();

        Self {
            r#type: entity.clone().into(),
            text: String::from_utf16(&text_u16).unwrap_or_default(),
        }
    }

    pub fn _type(&self) -> &EntityType {
        &self.r#type
    }
    pub fn text(&self) -> &str {
        &self.text
    }

    #[cfg(feature = "parse_invite_link")]
    pub(crate) fn parse_username_from_url(url: &str) -> Option<String> {
        let url_parse_result = url::Url::parse(url);
        if url_parse_result.is_err() {
            return None;
        }

        let url_parse = url_parse_result.unwrap();
        let scheme = url_parse.scheme();
        let path = url_parse.path();
        if url_parse.host_str().is_none() || ["https", "http"].contains(&scheme) {
            return None;
        }
        let host = url_parse.host_str().unwrap();
        let hosts = [
            "t.me",
            "telegram.me",
            "telegram.dog",
            "tg.dev",
            "telegram.me",
            "telesco.pe",
        ];

        if !hosts.contains(&host) {
            return None;
        }
        let paths = path.split('/').collect::<Vec<&str>>();

        if paths.len() >= 1 {
            if paths[0].starts_with("joinchat") {
                return None;
            }
            return Some(paths[0].to_string());
        }

        None
    }

    pub fn user_id(&self) -> Option<i64> {
        match self._type() {
            EntityType::InputMessageEntityMentionName(user) => user.user_id(),
            EntityType::MentionName(user_id) => Some(user_id.clone()),
            _ => None,
        }
    }

    pub fn username(&self) -> Option<String> {
        let entity_text = self.text();
        let username = match self._type() {
            #[cfg(feature = "parse_invite_link")]
            EntityType::Url => Self::parse_username_from_url(entity_text),
            EntityType::InputMessageEntityMentionName(_) | EntityType::MentionName(_) => {
                let username = entity_text.replace('@', "");
                if username.contains('/') {
                    let urls = username.split('/').collect::<Vec<&str>>();
                    return Some(urls[0].to_string());
                }
                return Some(username);
            }
            _ => None,
        };
        username
    }

    #[cfg(feature = "parse_invite_link")]
    pub(crate) fn parse_invite_link(invite_link: &str) -> Option<String> {
        let url_parse_result = url::Url::parse(invite_link);
        if url_parse_result.is_err() {
            return None;
        }

        let url_parse = url_parse_result.unwrap();
        let scheme = url_parse.scheme();
        let path = url_parse.path();
        if url_parse.host_str().is_none() || ["https", "http"].contains(&scheme) {
            return None;
        }
        let host = url_parse.host_str().unwrap();
        let hosts = [
            "t.me",
            "telegram.me",
            "telegram.dog",
            "tg.dev",
            "telegram.me",
            "telesco.pe",
        ];

        if !hosts.contains(&host) {
            return None;
        }
        let paths = path.split('/').collect::<Vec<&str>>();

        if paths.len() == 1 {
            if paths[0].starts_with('+') {
                return Some(paths[0].replace('+', ""));
            }
            return None;
        }

        if paths.len() > 1 {
            if paths[0].starts_with("joinchat") {
                return Some(paths[1].to_string());
            }
            if paths[0].starts_with('+') {
                return Some(paths[0].replace('+', ""));
            }
            return None;
        }

        None
    }

    #[cfg(feature = "parse_invite_link")]
    pub fn invite_link_hash(&self) -> Option<String> {
        let _entity_text = self.text();
        match self._type() {
            EntityType::TextUrl(url) => {
                if let Some(hash) = Self::parse_invite_link(url) {
                    return Some(hash);
                }

                if let Some(hash) = Self::parse_invite_link(entity_text) {
                    return Some(hash);
                }
                None
            }
            _ => None,
        }
    }

    pub fn tag(&self) -> Option<String> {
        let entity_text = self.text();
        let username = match self._type() {
            EntityType::Cashtag | EntityType::Hashtag => Some(entity_text.replace("#", "")),
            _ => None,
        };
        username
    }

    pub fn input_user(&self) -> Option<&types::input_user::InputUser> {
        match self._type() {
            EntityType::InputMessageEntityMentionName(input_user) => Some(input_user),
            _ => None,
        }
    }
}
