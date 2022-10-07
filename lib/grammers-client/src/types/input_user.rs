use grammers_tl_types as tl;
pub struct InputUser(tl::enums::InputUser);

impl InputUser {
    pub fn from(raw: tl::enums::InputUser) -> Self {
        Self(raw)
    }

    pub fn new_user_id(user_id: i64, access_hash: i64) -> tl::enums::InputUser {
        tl::types::InputUser {
            user_id,
            access_hash,
        }
        .into()
    }
}
