// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use grammers_tl_types as tl;
use std::fmt::{self, Write as _};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PackedType {
    // The fancy bit pattern may enable some optimizations.
    // * 2nd bit for tl::enums::Peer::User
    // * 3rd bit for tl::enums::Peer::Chat
    // * 6th bit for tl::enums::Peer::Channel
    User = 0b0000_0010,
    Bot = 0b0000_0011,
    Chat = 0b0000_0100,
    Megagroup = 0b0010_1000,
    Broadcast = 0b0011_0000,
    Gigagroup = 0b0011_1000,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A packed chat
pub struct PackedChat {
    pub ty: PackedType,
    pub id: i64,
    pub access_hash: Option<i64>,
}

impl PackedChat {
    /// Serialize the packed chat into a new buffer and return its bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = if let Some(access_hash) = self.access_hash {
            let mut res = vec![0; 18];
            res[10..18].copy_from_slice(&access_hash.to_le_bytes());
            res
        } else {
            vec![0; 10]
        };
        res[0] = self.ty as u8;
        res[1] = res.len() as u8;
        res[2..10].copy_from_slice(&self.id.to_le_bytes());
        res
    }

    /// Serialize the packed chat as an hexadecimal string.
    pub fn to_hex(&self) -> String {
        let bytes = self.to_bytes();
        let mut result = String::with_capacity(bytes.len() * 2);
        bytes.into_iter().for_each(|b| {
            write!(result, "{:02x}", b).unwrap();
        });
        result
    }

    /// Deserialize the buffer into a packed chat.
    #[allow(clippy::result_unit_err)]
    pub fn from_bytes(buf: &[u8]) -> Result<Self, ()> {
        if buf.len() != 10 && buf.len() != 18 {
            return Err(());
        }
        if buf[1] as usize != buf.len() {
            return Err(());
        }
        let ty = match buf[0] {
            0b0000_0010 => PackedType::User,
            0b0000_0011 => PackedType::Bot,
            0b0000_0100 => PackedType::Chat,
            0b0010_1000 => PackedType::Megagroup,
            0b0011_0000 => PackedType::Broadcast,
            0b0011_1000 => PackedType::Gigagroup,
            _ => return Err(()),
        };
        let id = i64::from_le_bytes([
            buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9],
        ]);
        let access_hash = if buf[1] == 18 {
            Some(i64::from_le_bytes([
                buf[10], buf[11], buf[12], buf[13], buf[14], buf[15], buf[16], buf[17],
            ]))
        } else {
            None
        };
        Ok(Self {
            ty,
            id,
            access_hash,
        })
    }

    /// Deserialize the hexadecimal string into a packed chat.
    pub fn from_hex(hex: &str) -> Result<Self, ()> {
        fn hex_to_decimal(hex_digit: u8) -> Option<u8> {
            Some(match hex_digit {
                b'0'..=b'9' => hex_digit - b'0',
                b'a'..=b'f' => hex_digit - b'a' + 0xa,
                b'A'..=b'F' => hex_digit - b'A' + 0xa,
                _ => return None,
            })
        }

        if hex.len() % 2 != 0 {
            return Err(());
        }

        let bytes = hex
            .as_bytes()
            .chunks_exact(2)
            .map(
                |slice| match (hex_to_decimal(slice[0]), hex_to_decimal(slice[1])) {
                    (Some(h), Some(l)) => Ok(h * 0x10 + l),
                    _ => Err(()),
                },
            )
            .collect::<Result<Vec<_>, _>>()?;

        Self::from_bytes(&bytes)
    }

    pub fn is_user(&self) -> bool {
        matches!(self.ty, PackedType::User | PackedType::Bot)
    }

    pub fn is_chat(&self) -> bool {
        matches!(self.ty, PackedType::Chat)
    }

    pub fn is_channel(&self) -> bool {
        matches!(
            self.ty,
            PackedType::Megagroup | PackedType::Broadcast | PackedType::Gigagroup
        )
    }

    pub fn to_peer(&self) -> tl::enums::Peer {
        match self.ty {
            PackedType::User | PackedType::Bot => tl::types::PeerUser { user_id: self.id }.into(),
            PackedType::Chat => tl::types::PeerChat { chat_id: self.id }.into(),
            PackedType::Megagroup | PackedType::Broadcast | PackedType::Gigagroup => {
                tl::types::PeerChannel {
                    channel_id: self.id,
                }
                .into()
            }
        }
    }

    pub fn to_input_peer(&self) -> tl::enums::InputPeer {
        match self.ty {
            PackedType::User | PackedType::Bot => tl::types::InputPeerUser {
                user_id: self.id,
                access_hash: self.access_hash.unwrap_or(0),
            }
            .into(),
            PackedType::Chat => tl::types::InputPeerChat { chat_id: self.id }.into(),
            PackedType::Megagroup | PackedType::Broadcast | PackedType::Gigagroup => {
                tl::types::InputPeerChannel {
                    channel_id: self.id,
                    access_hash: self.access_hash.unwrap_or(0),
                }
                .into()
            }
        }
    }

    pub fn try_to_input_user(&self) -> Option<tl::enums::InputUser> {
        match self.ty {
            PackedType::User | PackedType::Bot => Some(
                tl::types::InputUser {
                    user_id: self.id,
                    access_hash: self.access_hash.unwrap_or(0),
                }
                .into(),
            ),
            _ => None,
        }
    }

    pub fn to_input_user_lossy(&self) -> tl::enums::InputUser {
        self.try_to_input_user().unwrap_or_else(|| {
            tl::types::InputUser {
                user_id: 0,
                access_hash: 0,
            }
            .into()
        })
    }

    pub fn try_to_chat_id(&self) -> Option<i64> {
        match self.ty {
            PackedType::Chat => Some(self.id),
            _ => None,
        }
    }

    pub fn try_to_input_channel(&self) -> Option<tl::enums::InputChannel> {
        match self.ty {
            PackedType::Megagroup | PackedType::Broadcast | PackedType::Gigagroup => Some(
                tl::types::InputChannel {
                    channel_id: self.id,
                    access_hash: self.access_hash.unwrap_or(0),
                }
                .into(),
            ),
            _ => None,
        }
    }

    pub fn to_input_channel_lossy(&self) -> tl::enums::InputChannel {
        self.try_to_input_channel().unwrap_or_else(|| {
            tl::types::InputChannel {
                channel_id: 0,
                access_hash: 0,
            }
            .into()
        })
    }
}

impl fmt::Display for PackedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::User => "User",
            Self::Bot => "Bot",
            Self::Chat => "Group",
            Self::Megagroup => "Supergroup",
            Self::Broadcast => "Channel",
            Self::Gigagroup => "BroadcastGroup",
        })
    }
}

impl fmt::Display for PackedChat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PackedChat::{}({})", self.ty, self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_hex_reciprocal() {
        let pc = PackedChat {
            ty: PackedType::User,
            id: 123,
            access_hash: Some(456789),
        };
        assert_eq!(PackedChat::from_hex(&pc.to_hex()), Ok(pc));

        let pc = PackedChat {
            ty: PackedType::Chat,
            id: 987,
            access_hash: None,
        };
        assert_eq!(PackedChat::from_hex(&pc.to_hex()), Ok(pc));
    }
}
