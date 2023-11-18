use grammers_tl_types as tl;
pub struct InputPeer(pub(crate) tl::enums::InputPeer);

impl InputPeer {
    pub fn from(raw: tl::enums::InputPeer) -> Self {
        Self(raw)
    }
}

#[cfg(feature = "unstable_raw")]
impl From<InputUser> for tl::enums::InputUser {
    fn from(input_user: InputUser) -> Self {
        input_user.0
    }
}
