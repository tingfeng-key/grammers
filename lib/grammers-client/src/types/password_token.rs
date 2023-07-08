// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use grammers_tl_types as tl;
use tl::types::PasswordKdfAlgoSha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow;

// TODO this should not be Clone, but check_password Err doesn't include it back yet
#[derive(Clone, Debug)]
pub struct PasswordToken {
    pub(crate) password: tl::types::account::Password,
}

impl PasswordToken {
    pub fn new(password: tl::types::account::Password) -> Self {
        PasswordToken { password }
    }

    pub fn hint(self) -> String {
        self.password.hint.unwrap_or_default()
    }

    pub fn has_password(&self) -> bool {
        self.password.has_password
    }

    fn srp_id(&self) -> i64 {
        self.password.srp_id.unwrap_or_default()
    }

    fn srp_b(&self) -> Vec<u8> {
        self.password.srp_b.clone().unwrap_or_default()
    }

    fn secure_random(&self) -> Vec<u8> {
        self.password.secure_random.clone()
    }

    fn current_algo(
        &self,
    ) -> Option<PasswordKdfAlgoSha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow> {
        use tl::enums::PasswordKdfAlgo::{
            Sha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow, Unknown,
        };
        match self.password.current_algo.clone() {
            Some(algo) => match algo {
                Unknown => None,
                Sha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow(a) => Some(a),
            },
            None => None,
        }
    }

    fn new_algo(
        &self,
    ) -> Option<PasswordKdfAlgoSha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow> {
        use tl::enums::PasswordKdfAlgo::{
            Sha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow, Unknown,
        };
        match self.password.new_algo.clone() {
            Unknown => None,
            Sha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow(a) => Some(a),
        }
    }

    pub fn to_fa(&self, current_password: String) -> (Vec<u8>, Vec<u8>) {
        use grammers_crypto::two_factor_auth::calculate_2fa;
        let current_algo = self.current_algo().unwrap();
        calculate_2fa(
            &current_algo.salt1,
            &current_algo.salt2,
            &current_algo.g,
            &current_algo.p,
            self.srp_b(),
            self.secure_random(),
            current_password,
        )
    }

    pub fn to_input_check_password_srp(
        &self,
        current_password: String,
    ) -> tl::enums::InputCheckPasswordSrp {
        let (m1, a) = self.to_fa(current_password);
        tl::types::InputCheckPasswordSrp {
            srp_id: self.srp_id(),
            a,
            m1,
        }
        .into()
    }

    pub fn generate_new_hash(
        &self,
        new_password: String,
    ) -> Option<(
        PasswordKdfAlgoSha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow,
        Vec<u8>,
    )> {
        use grammers_crypto::two_factor_auth::{compute_password_hash, generate_random_32_bytes};
        println!("{:#?}", self);
        match self.new_algo() {
            Some(mut new_algo) => {
                let rand = generate_random_32_bytes();
                // println!("{:#?}", new_algo.g);
                // println!("{:#?}", new_algo.p);
                // println!("{:#?}", new_algo.salt1);
                // println!("{:#?}", new_algo.salt2);
                // println!("{:#?}", rand);
                new_algo.salt1.extend_from_slice(&rand);
                let new_password_hash = compute_password_hash(
                    &new_algo.salt1,
                    &new_algo.salt2,
                    &new_algo.g,
                    &new_algo.p,
                    new_password,
                );
                // println!("{:#?}", new_password_hash);
                Some((new_algo.clone(), new_password_hash))
            }
            None => None,
        }
    }
}
