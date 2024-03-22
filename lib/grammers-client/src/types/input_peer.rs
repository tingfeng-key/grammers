use grammers_tl_types as tl;
#[allow(dead_code)]
pub struct InputPeer(pub(crate) tl::enums::InputPeer);

impl InputPeer {
    pub fn _from_raw(raw: tl::enums::InputPeer) -> Self {
        Self(raw)
    }
}

#[cfg(feature = "unstable_raw")]
impl From<InputPeer> for tl::enums::InputPeer {
    fn from(input_user: InputPeer) -> Self {
        input_user.0
    }
}
