use grammers_tl_types as tl;
pub struct InputPeer(pub(crate) tl::enums::InputPeer);

impl InputPeer {
    pub fn from(raw: tl::enums::InputPeer) -> Self {
        Self(raw)
    }
}
