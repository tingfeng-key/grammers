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
        self.password.srp_id.unwrap()
    }

    pub fn algo(
        &self,
        is_new: bool,
    ) -> PasswordKdfAlgoSha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow {
        let pwd = self.password.clone();
        let mut current_algo = pwd.new_algo;
        if !is_new {
            current_algo = pwd.current_algo.unwrap();
        }
        let params = crate::utils::extract_password_parameters(&current_algo);
        if !grammers_crypto::two_factor_auth::check_p_and_g(params.2, params.3) {
            panic!("Failed to get correct password information from Telegram")
        }
        let (salt1, salt2, p, g) = params;
        let mut new_salt1 = salt1.clone();
        if is_new {
            let none = &grammers_crypto::two_factor_auth::generate_random_32_bytes();
            new_salt1.extend_from_slice(none);
        }

        PasswordKdfAlgoSha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow {
            salt1: new_salt1,
            salt2: salt2.clone(),
            g: *g,
            p: p.clone(),
        }
    }

    pub async fn to_input_check_password_srp(
        &self,
        algo: PasswordKdfAlgoSha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow,
        current_password: &str,
    ) -> tl::enums::InputCheckPasswordSrp {
        let g_b = self.password.srp_b.clone().unwrap();
        let a = self.password.secure_random.clone();

        let (m1, a) = grammers_crypto::two_factor_auth::calculate_2fa(
            &algo.salt1,
            &algo.salt2,
            &algo.p,
            &algo.g,
            g_b,
            a,
            current_password,
        );
        tl::types::InputCheckPasswordSrp {
            srp_id: self.srp_id(),
            a: a.to_vec(),
            m1: m1.to_vec(),
        }
        .into()
    }

    pub fn generate_new_hash(
        &self,
        new_algo: PasswordKdfAlgoSha256Sha256Pbkdf2Hmacsha512iter100000Sha256ModPow,
        new_password: &str,
    ) -> Vec<u8> {
        grammers_crypto::two_factor_auth::compute_password_hash(
            &new_algo.salt1,
            &new_algo.salt2,
            &new_algo.g,
            &new_algo.p,
            new_password,
        )
        .to_vec()
    }
}
