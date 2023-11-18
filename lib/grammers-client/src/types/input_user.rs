use grammers_tl_types as tl;

#[derive(Debug)]
pub struct InputUser(pub(crate) tl::enums::InputUser);

impl InputUser {
    pub fn _from_raw(raw: tl::enums::InputUser) -> Self {
        Self(raw)
    }

    pub fn _from_message(peer: tl::enums::InputPeer, msg_id: i32, user_id: i64) -> Self {
        Self::_from_raw(
            tl::types::InputUserFromMessage {
                peer,
                msg_id,
                user_id,
            }
            .into(),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.0 == tl::enums::InputUser::Empty
    }

    pub fn is_self(&self) -> bool {
        self.0 == tl::enums::InputUser::UserSelf
    }

    pub fn user_id(&self) -> Option<i64> {
        match &self.0 {
            tl::enums::InputUser::User(user) => Some(user.user_id),
            tl::enums::InputUser::Empty | tl::enums::InputUser::UserSelf => None,
            tl::enums::InputUser::FromMessage(input_user_from_message) => {
                Some(input_user_from_message.user_id)
            }
        }
    }

    pub fn user_access_hash(&self) -> Option<i64> {
        match &self.0 {
            tl::enums::InputUser::User(user) => Some(user.access_hash),
            tl::enums::InputUser::Empty
            | tl::enums::InputUser::UserSelf
            | tl::enums::InputUser::FromMessage(_) => None,
        }
    }

    pub fn message_id(&self) -> Option<i32> {
        match &self.0 {
            tl::enums::InputUser::FromMessage(input_user_from_message) => {
                Some(input_user_from_message.msg_id)
            }
            tl::enums::InputUser::Empty
            | tl::enums::InputUser::UserSelf
            | tl::enums::InputUser::User(_) => None,
        }
    }

    pub fn message_peer(&self) -> Option<tl::enums::InputPeer> {
        match &self.0 {
            tl::enums::InputUser::FromMessage(input_user_from_message) => {
                Some(input_user_from_message.peer.clone())
            }
            tl::enums::InputUser::Empty
            | tl::enums::InputUser::UserSelf
            | tl::enums::InputUser::User(_) => None,
        }
    }
}

#[cfg(feature = "unstable_raw")]
impl From<InputUser> for tl::enums::InputUser {
    fn from(input_user: InputUser) -> Self {
        input_user.0
    }
}
