use grammers_tl_types as tl;
pub struct InputUser(pub(crate) tl::enums::InputUser);

impl InputUser {
    pub fn from(raw: tl::enums::InputUser) -> Self {
        Self(raw)
    }

    pub fn from_message(peer: tl::enums::InputPeer, msg_id: i32, user_id: i64) -> Self {
        Self::from(
            tl::types::InputUserFromMessage {
                peer,
                msg_id,
                user_id,
            }
            .into(),
        )
    }

    pub fn to_raw(self) -> tl::enums::InputUser {
        self.0
    }
}
