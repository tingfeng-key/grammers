use grammers_tl_types as tl;
pub struct InputChannel(tl::enums::InputChannel);

impl InputChannel {
    pub fn from(raw: tl::enums::InputChannel) -> Self {
        Self(raw)
    }
    pub fn new_empty() -> Self {
        Self(tl::types::InputChannelEmpty {}.into())
    }

    pub fn new_channel(channel_id: i64, access_hash: i64) -> tl::enums::InputChannel {
        tl::types::InputChannel {
            channel_id,
            access_hash,
        }
        .into()
    }

    pub fn new_channel_from_msg(
        channel_id: i64,
        peer: tl::enums::InputPeer,
        msg_id: i32,
    ) -> tl::enums::InputChannel {
        tl::types::InputChannelFromMessage {
            channel_id,
            peer,
            msg_id,
        }
        .into()
    }
}
