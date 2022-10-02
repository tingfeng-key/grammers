// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
