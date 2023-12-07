// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use grammers_tl_types::deserialize::Result;

use crate::Client;

#[derive(Debug, Clone)]
pub struct LoginToken {
    pub(crate) phone: String,
    pub(crate) phone_code_hash: String,
}

impl LoginToken {
    pub fn build(phone: String, phone_code_hash: String) -> Self {
        Self {
            phone,
            phone_code_hash,
        }
    }
    pub fn phone(&self) -> &str {
        self.phone.as_str()
    }

    pub fn phone_code_hash(&self) -> &str {
        self.phone_code_hash.as_ref()
    }
}

pub struct QrToken {
    pub(crate) token: Vec<u8>,
    pub(crate) expires: i32,
    pub(crate) except_ids: Vec<i64>,
    pub(crate) client: Client,
}

impl QrToken {
    pub fn token(&self) -> &[u8] {
        self.token.as_ref()
    }
}
