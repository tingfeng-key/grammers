// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use glass_pumpkin::safe_prime;
use hmac::Hmac;
use num_bigint::BigUint;
use sha2::digest::Output;
use sha2::{Digest, Sha256, Sha512};

// H(data) := sha256(data)
macro_rules! h {
    ( $( $x:expr ),* ) => {
        {
            let mut hasher = Sha256::new();
            $(
                hasher.update($x);
            )*
            hasher.finalize()
        }
    };
}

/// Prepare the password for sending to telegram for verification.
/// The method returns M1 and g_a parameters that should be sent to Telegram
/// (without the raw password).
///
/// The algorithm is described in <https://core.telegram.org/api/srp>.
pub fn calculate_2fa(
    salt1: &[u8],
    salt2: &[u8],
    g: &i32,
    p: &[u8],
    g_b: Vec<u8>,
    a: Vec<u8>,
    password: impl AsRef<[u8]>,
) -> (Vec<u8>, Vec<u8>) {
    // Prepare our parameters
    let big_p = BigUint::from_bytes_be(p);

    let g_b = pad_to_256(&g_b);
    let a = pad_to_256(&a);

    let g_for_hash = vec![*g as u8];
    let g_for_hash = pad_to_256(&g_for_hash);

    let big_g_b = BigUint::from_bytes_be(&g_b);

    let big_g = BigUint::from(*g as u32);
    let big_a = BigUint::from_bytes_be(&a);

    // k := H(p | g)
    let k = h!(&p, &g_for_hash);
    let big_k = BigUint::from_bytes_be(&k);

    // g_a := pow(g, a) mod p
    let g_a = big_g.modpow(&big_a, &big_p);
    let g_a = pad_to_256(&g_a.to_bytes_be());

    // u := H(g_a | g_b)
    let u = h!(&g_a, &g_b);
    let u = BigUint::from_bytes_be(&u);

    // x := PH2(password, salt1, salt2)
    let x = ph2(&password, salt1, salt2);
    let x = BigUint::from_bytes_be(&x);

    // v := pow(g, x) mod p
    let big_v = big_g.modpow(&x, &big_p);

    // k_v := (k * v) mod p
    let k_v = (big_k * big_v) % &big_p;

    // t := (g_b - k_v) mod p (positive modulo, if the result is negative increment by p)
    let sub = if big_g_b > k_v {
        big_g_b - k_v
    } else {
        k_v - big_g_b
    };
    let big_t = sub % &big_p;

    // s_a := pow(t, a + u * x) mod p
    let first = u * x;
    let second = big_a + first;
    let big_s_a = big_t.modpow(&(second), &big_p);

    // k_a := H(s_a)
    let k_a = h!(&pad_to_256(&big_s_a.to_bytes_be()));

    // M1 := H(H(p) xor H(g) | H(salt1) | H(salt2) | g_a | g_b | k_a)
    let h_p = h!(&p);
    let h_g = h!(&g_for_hash);

    let p_xor_g: Vec<u8> = xor(&h_p, &h_g);

    let m1 = h!(&p_xor_g, &h!(&salt1), &h!(&salt2), &g_a, &g_b, &k_a).to_vec();

    (m1, g_a)
}

/// Validation for parameters required for two-factor authentication
pub fn check_p_and_g(g: &i32, p: &[u8]) -> bool {
    if !check_p_len(p) {
        return false;
    }

    check_p_prime_and_subgroup(p, g)
}

fn check_p_prime_and_subgroup(p: &[u8], g: &i32) -> bool {
    let p = &BigUint::from_bytes_be(p);

    if !safe_prime::check(p) {
        return false;
    }

    match g {
        2 => p % 8u8 == BigUint::from(7u8),
        3 => p % 3u8 == BigUint::from(2u8),
        4 => true,
        5 => {
            let mod_value = p % 5u8;
            mod_value == BigUint::from(1u8) || mod_value == BigUint::from(4u8)
        }
        6 => {
            let mod_value = p % 24u8;
            mod_value == BigUint::from(19u8) || mod_value == BigUint::from(23u8)
        }
        7 => {
            let mod_value = p % 7u8;
            mod_value == BigUint::from(3u8)
                || mod_value == BigUint::from(5u8)
                || mod_value == BigUint::from(6u8)
        }
        _ => panic!("Unexpected g parameter"),
    }
}

fn check_p_len(p: &[u8]) -> bool {
    p.len() == 256
}

// SH(data, salt) := H(salt | data | salt)
fn sh(data: impl AsRef<[u8]>, salt: impl AsRef<[u8]>) -> Output<Sha256> {
    return h!(&salt, &data, &salt);
}

// PH1(password, salt1, salt2) := SH(SH(password, salt1), salt2)
fn ph1(password: impl AsRef<[u8]>, salt1: &[u8], salt2: &[u8]) -> Output<Sha256> {
    sh(&sh(password, salt1), salt2)
}

// PH2(password, salt1, salt2)
//                      := SH(pbkdf2(sha512, PH1(password, salt1, salt2), salt1, 100000), salt2)
fn ph2(password: impl AsRef<[u8]>, salt1: &[u8], salt2: &[u8]) -> Output<Sha256> {
    let hash1 = ph1(password, salt1, salt2);

    // 512-bit derived key
    let mut dk = [0u8; 64];
    pbkdf2::pbkdf2::<Hmac<Sha512>>(&hash1, salt1, 100000, &mut dk);

    sh(&dk, salt2)
}

fn xor(left: &Output<Sha256>, right: &Output<Sha256>) -> Vec<u8> {
    return left
        .iter()
        .zip(right.iter())
        .map(|(&x1, &x2)| x1 ^ x2)
        .collect();
}

fn pad_to_256(data: &[u8]) -> Vec<u8> {
    let mut new_vec = vec![0; 256 - data.len()];
    new_vec.extend(data);
    new_vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_calculations_1() {
        let salt1 = vec![1];
        let salt2 = vec![2];
        let g = 3;
        let p = pad_to_256(&vec![47]);
        let g_b = vec![5];
        let a = vec![6];
        let password = vec![7];

        let (m1, g_a) = calculate_2fa(&salt1, &salt2, &g, &p, g_b, a, password);

        let expected_m1 = vec![
            157, 131, 196, 103, 0, 184, 116, 232, 7, 196, 85, 231, 17, 36, 30, 222, 158, 234, 98,
            88, 59, 56, 71, 215, 183, 123, 122, 50, 19, 32, 54, 206,
        ];
        let expected_g_a = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24,
        ];

        assert_eq!(expected_m1, m1);
        assert_eq!(expected_g_a, g_a);
    }

    #[test]
    fn check_calculations_2() {
        let salt1 = vec![
            0x5f, 0x48, 0x3c, 0x38, 0xbd, 0x9, 0x86, 0xe7, 0xcd, 0xc9, 0x5a, 0xe1, 0x38, 0xef,
            0x4f, 0x49, 0xb9, 0x51, 0xc1, 0xf8, 0x1c, 0x71, 0x3f, 0xec, 0xde, 0xf3, 0xaf, 0x69,
            0x2c, 0xec, 0x4b, 0x47, 0x16, 0xac, 0x9b, 0x77, 0xa, 0x19, 0x5e, 0xbe,
        ];
        let salt2 = vec![
            0xb6, 0x16, 0xfc, 0x6b, 0xbe, 0xdf, 0x51, 0x11, 0x19, 0xc5, 0xed, 0x34, 0x62, 0x95,
            0x27, 0xf1,
        ];
        let g = 3;
        let p = vec![
            0xc7, 0x1c, 0xae, 0xb9, 0xc6, 0xb1, 0xc9, 0x4, 0x8e, 0x6c, 0x52, 0x2f, 0x70, 0xf1,
            0x3f, 0x73, 0x98, 0xd, 0x40, 0x23, 0x8e, 0x3e, 0x21, 0xc1, 0x49, 0x34, 0xd0, 0x37,
            0x56, 0x3d, 0x93, 0xf, 0x48, 0x19, 0x8a, 0xa, 0xa7, 0xc1, 0x40, 0x58, 0x22, 0x94, 0x93,
            0xd2, 0x25, 0x30, 0xf4, 0xdb, 0xfa, 0x33, 0x6f, 0x6e, 0xa, 0xc9, 0x25, 0x13, 0x95,
            0x43, 0xae, 0xd4, 0x4c, 0xce, 0x7c, 0x37, 0x20, 0xfd, 0x51, 0xf6, 0x94, 0x58, 0x70,
            0x5a, 0xc6, 0x8c, 0xd4, 0xfe, 0x6b, 0x6b, 0x13, 0xab, 0xdc, 0x97, 0x46, 0x51, 0x29,
            0x69, 0x32, 0x84, 0x54, 0xf1, 0x8f, 0xaf, 0x8c, 0x59, 0x5f, 0x64, 0x24, 0x77, 0xfe,
            0x96, 0xbb, 0x2a, 0x94, 0x1d, 0x5b, 0xcd, 0x1d, 0x4a, 0xc8, 0xcc, 0x49, 0x88, 0x7, 0x8,
            0xfa, 0x9b, 0x37, 0x8e, 0x3c, 0x4f, 0x3a, 0x90, 0x60, 0xbe, 0xe6, 0x7c, 0xf9, 0xa4,
            0xa4, 0xa6, 0x95, 0x81, 0x10, 0x51, 0x90, 0x7e, 0x16, 0x27, 0x53, 0xb5, 0x6b, 0xf,
            0x6b, 0x41, 0xd, 0xba, 0x74, 0xd8, 0xa8, 0x4b, 0x2a, 0x14, 0xb3, 0x14, 0x4e, 0xe, 0xf1,
            0x28, 0x47, 0x54, 0xfd, 0x17, 0xed, 0x95, 0xd, 0x59, 0x65, 0xb4, 0xb9, 0xdd, 0x46,
            0x58, 0x2d, 0xb1, 0x17, 0x8d, 0x16, 0x9c, 0x6b, 0xc4, 0x65, 0xb0, 0xd6, 0xff, 0x9c,
            0xa3, 0x92, 0x8f, 0xef, 0x5b, 0x9a, 0xe4, 0xe4, 0x18, 0xfc, 0x15, 0xe8, 0x3e, 0xbe,
            0xa0, 0xf8, 0x7f, 0xa9, 0xff, 0x5e, 0xed, 0x70, 0x5, 0xd, 0xed, 0x28, 0x49, 0xf4, 0x7b,
            0xf9, 0x59, 0xd9, 0x56, 0x85, 0xc, 0xe9, 0x29, 0x85, 0x1f, 0xd, 0x81, 0x15, 0xf6, 0x35,
            0xb1, 0x5, 0xee, 0x2e, 0x4e, 0x15, 0xd0, 0x4b, 0x24, 0x54, 0xbf, 0x6f, 0x4f, 0xad,
            0xf0, 0x34, 0xb1, 0x4, 0x3, 0x11, 0x9c, 0xd8, 0xe3, 0xb9, 0x2f, 0xcc, 0x5b,
        ];
        let g_b = vec![
            0x93, 0xf7, 0xe, 0xbd, 0x50, 0xf6, 0x42, 0x6a, 0xca, 0x25, 0x68, 0x76, 0x95, 0x99,
            0xf9, 0x1f, 0x24, 0xd0, 0x12, 0x84, 0xcc, 0x51, 0xa4, 0x49, 0xe6, 0x2d, 0xcc, 0x15,
            0x27, 0xdf, 0xe5, 0x1, 0x26, 0xb2, 0xaa, 0x44, 0x23, 0x8e, 0x4f, 0xb2, 0x33, 0x14,
            0x19, 0xed, 0x4a, 0xeb, 0xf1, 0xa0, 0xae, 0x15, 0xe0, 0x3a, 0xbd, 0x18, 0xbf, 0xc5,
            0x2c, 0xa6, 0xba, 0xec, 0x56, 0x4c, 0x13, 0xb5, 0xa1, 0xd2, 0xe3, 0x57, 0x79, 0x98,
            0x97, 0x75, 0x7b, 0xb7, 0x36, 0xfd, 0xc2, 0xce, 0xb1, 0xb5, 0x6a, 0xac, 0xf1, 0x9a,
            0xb3, 0x54, 0x8d, 0x6d, 0x92, 0x2a, 0x52, 0x2f, 0xb, 0x51, 0xf4, 0x1, 0x24, 0xc3, 0xbc,
            0x99, 0x36, 0xaf, 0xf3, 0xe1, 0xdc, 0xfb, 0xea, 0x39, 0xac, 0x9a, 0xd2, 0xad, 0xdc,
            0x6a, 0xf0, 0xad, 0x30, 0x78, 0x32, 0x78, 0xbb, 0xb8, 0x4c, 0xab, 0xe, 0xd8, 0x46,
            0x4b, 0xf, 0xfe, 0xb2, 0xb0, 0xc9, 0x3a, 0x39, 0xa5, 0xd9, 0x7d, 0xba, 0x1, 0x5, 0x67,
            0x2c, 0xa5, 0x47, 0x53, 0x73, 0xd8, 0xd2, 0x3e, 0x54, 0xa6, 0xac, 0x9b, 0xed, 0x95,
            0x19, 0xe8, 0xbe, 0xf4, 0xf0, 0x7, 0x19, 0xf5, 0xad, 0x56, 0x15, 0x1b, 0xe5, 0x53,
            0x76, 0x48, 0xdf, 0x2f, 0x8e, 0x3e, 0x72, 0x65, 0xcb, 0x57, 0xfb, 0x94, 0xa0, 0x54,
            0xce, 0x2a, 0x82, 0xb8, 0xcc, 0x66, 0x4a, 0xd0, 0x60, 0xe0, 0xd6, 0xc6, 0xdf, 0x18,
            0x79, 0x34, 0x54, 0x40, 0xeb, 0x97, 0x7f, 0xa0, 0xf2, 0xd3, 0x6f, 0x31, 0xa2, 0x53,
            0xd8, 0x91, 0x77, 0x32, 0xf1, 0x33, 0xd4, 0x0, 0x33, 0xa3, 0x4b, 0x61, 0x52, 0x96,
            0x9b, 0x60, 0xd, 0x59, 0xcd, 0xab, 0xfe, 0xa2, 0xab, 0x23, 0x93, 0xad, 0x65, 0x9e,
            0x56, 0xd6, 0x6e, 0x13, 0x60, 0x5b, 0x1f, 0x61, 0xe4, 0x8e, 0x3c, 0xd6, 0x5c, 0xf,
            0x58, 0xac,
        ];
        let a = vec![
            0xbf, 0x31, 0x57, 0x4f, 0x34, 0xfc, 0xe1, 0xe5, 0x38, 0x91, 0xc5, 0x9b, 0x7f, 0x62,
            0x46, 0x8a, 0xc, 0xa6, 0x82, 0xda, 0x85, 0x85, 0xdf, 0x8d, 0xe0, 0xa1, 0x88, 0x73,
            0x35, 0x97, 0x55, 0xfb, 0xb1, 0x81, 0x88, 0x78, 0xa9, 0xee, 0x91, 0x9b, 0xb1, 0xe9,
            0x4d, 0x20, 0xc5, 0xf0, 0x60, 0x7e, 0x2, 0xa3, 0x31, 0x76, 0x19, 0x9b, 0xf3, 0x22, 0x2,
            0x56, 0xc9, 0xea, 0x1a, 0x69, 0xf3, 0x95, 0xa5, 0x15, 0xd2, 0x5, 0x39, 0xd8, 0x8c,
            0xda, 0x75, 0xa, 0x52, 0x52, 0xfb, 0x86, 0x4f, 0x57, 0x3f, 0x2b, 0x3, 0x2f, 0x3b, 0x46,
            0x7d, 0x8, 0xb3, 0x4f, 0xd9, 0xc8, 0x9d, 0x1c, 0x5d, 0x6, 0x27, 0x8e, 0x11, 0x3e, 0x51,
            0xd4, 0xe8, 0x93, 0xc1, 0xc0, 0x27, 0x45, 0x5a, 0xf4, 0x65, 0x3f, 0x9, 0x66, 0x7, 0xa4,
            0x15, 0x6d, 0x94, 0xfb, 0x8e, 0x1d, 0xc7, 0xce, 0xe5, 0xbf, 0xe3, 0x28, 0x50, 0x98,
            0x2f, 0x94, 0x1a, 0xe4, 0x41, 0x4e, 0x83, 0xbf, 0x22, 0xdf, 0x56, 0x27, 0xb, 0x43,
            0xb7, 0xcc, 0xc4, 0x4c, 0x26, 0xd4, 0x8, 0x86, 0x46, 0x4d, 0xa8, 0xe3, 0x44, 0xa8,
            0x54, 0x7, 0x95, 0xb8, 0xf6, 0x9b, 0x8f, 0x50, 0x85, 0x52, 0xa7, 0x23, 0xcd, 0x69,
            0x31, 0xe1, 0xd6, 0x92, 0x4, 0xe8, 0xe9, 0xdc, 0x5, 0x6f, 0xa, 0x2a, 0x10, 0xa0, 0xd7,
            0x95, 0x1e, 0x35, 0x5f, 0x3e, 0xde, 0xf5, 0xa5, 0xe1, 0x8a, 0x90, 0x91, 0x29, 0x5a,
            0x51, 0xeb, 0x9d, 0xb1, 0xb, 0x8b, 0xd, 0x30, 0x48, 0x9c, 0x8d, 0x29, 0xbc, 0xc, 0xd8,
            0x6e, 0x97, 0x78, 0x1f, 0x5e, 0x30, 0xc5, 0xb6, 0xbf, 0xe7, 0xca, 0xf4, 0xaa, 0xe8,
            0x1b, 0x28, 0x2e, 0x65, 0x3a, 0xc4, 0x8a, 0xa1, 0xa8, 0xfd, 0xe7, 0x89, 0x72, 0x2b,
            0xc0, 0x4f, 0x43, 0x20, 0xcd, 0x9f, 0x86, 0x84, 0x9f, 0xe0, 0x5c, 0xa4,
        ];
        let password = vec![50, 51, 52, 53, 54, 55];

        let (m1, g_a) = calculate_2fa(&salt1, &salt2, &g, &p, g_b, a, password);

        let expected_m1 = vec![
            77, 122, 244, 18, 197, 162, 231, 177, 84, 103, 55, 107, 209, 24, 184, 83, 96, 78, 104,
            123, 49, 245, 28, 73, 128, 196, 215, 193, 135, 102, 19, 227,
        ];
        let expected_g_a = vec![
            15, 161, 43, 198, 85, 177, 24, 122, 40, 41, 106, 105, 174, 86, 93, 104, 39, 130, 224,
            206, 176, 90, 8, 156, 12, 65, 193, 220, 233, 131, 220, 127, 74, 83, 67, 78, 167, 142,
            6, 93, 158, 28, 182, 14, 66, 123, 68, 104, 164, 120, 6, 9, 254, 186, 47, 85, 101, 78,
            226, 239, 224, 174, 183, 46, 218, 253, 226, 101, 46, 38, 237, 91, 77, 75, 170, 217,
            210, 163, 129, 128, 106, 246, 52, 22, 191, 98, 99, 223, 69, 164, 61, 133, 190, 84, 1,
            188, 34, 62, 191, 172, 9, 66, 28, 173, 221, 126, 38, 11, 214, 184, 101, 66, 19, 60, 4,
            141, 108, 213, 75, 56, 216, 226, 204, 223, 107, 85, 14, 135, 91, 19, 83, 164, 172, 254,
            50, 146, 255, 181, 106, 15, 88, 178, 163, 144, 39, 201, 191, 221, 145, 253, 76, 83, 29,
            35, 199, 125, 110, 143, 125, 88, 62, 174, 234, 49, 109, 237, 222, 214, 153, 53, 41,
            108, 231, 234, 52, 233, 190, 110, 242, 251, 216, 41, 234, 196, 201, 189, 100, 109, 193,
            86, 62, 71, 247, 123, 145, 67, 28, 160, 2, 207, 121, 252, 20, 157, 150, 130, 114, 131,
            92, 21, 202, 28, 107, 44, 94, 3, 113, 42, 46, 27, 93, 82, 245, 228, 250, 161, 193, 108,
            177, 119, 250, 253, 150, 166, 170, 91, 75, 63, 76, 100, 153, 21, 100, 75, 99, 133, 92,
            251, 56, 86, 31, 241, 127, 237, 251, 138,
        ];

        assert_eq!(expected_m1, m1);
        assert_eq!(expected_g_a, g_a);
    }

    #[test]
    fn test_check_p_and_g() {
        // Not prime
        assert_incorrect_pg(4, 0);
        // Bad prime
        assert_incorrect_pg(13, 0);

        assert_incorrect_pg(11, 2);
        assert_correct_pg(23, 2);

        assert_incorrect_pg(13, 3);
        assert_correct_pg(47, 3);

        assert_correct_pg(11, 4);

        assert_incorrect_pg(13, 5);
        assert_correct_pg(11, 5);
        assert_correct_pg(179, 5);

        assert_incorrect_pg(13, 6);
        assert_correct_pg(383, 6);

        assert_incorrect_pg(13, 7);
        assert_correct_pg(479, 7);
        assert_correct_pg(383, 7);
        assert_correct_pg(503, 7);
    }

    fn assert_incorrect_pg(p: u32, g: i32) {
        assert!(!check_p_prime_and_subgroup(&p.to_be_bytes().to_vec(), &g))
    }

    fn assert_correct_pg(p: u32, g: i32) {
        assert!(check_p_prime_and_subgroup(&p.to_be_bytes().to_vec(), &g))
    }
}
