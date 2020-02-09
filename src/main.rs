use crate::KeyEdition::{Business, Extreme, Engineer, NetworkAudit};
use core::convert::TryFrom;
use std::string::String;
use rand::{thread_rng, Rng};

#[derive(Debug, Copy, Clone)]
struct Date {
    day: i32,
    month: i32,
    year: i32
}

impl Date {
    fn enc(&mut self) -> i32 {
        self.year = self.year.max(2003).min(2099) - 2003;
        self.month = self.month.max(2).min(12);
        self.day = self.day.max(1).min(31);
        (self.year * 512) + (self.month * 32) + self.day
    }

    fn dec(val: i32) -> Date {
        Date {
            day: val & 31,
            month: (val >> 5) & 15,
            year: ((val >> 9) & 31) + 2003
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum KeyEdition {
    Business = 0,
    Extreme = 1,
    Engineer = 2,
    NetworkAudit = 3
}

impl TryFrom<i32> for KeyEdition {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Business),
            1 => Ok(Extreme),
            2 => Ok(Engineer),
            3 => Ok(NetworkAudit),
            _ => Err("Unknown edition"),
        }
    }
}

impl TryFrom<&str> for KeyEdition {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "business" => Ok(Business),
            "extreme" => Ok(Extreme),
            "engineer" => Ok(Engineer),
            "network" => Ok(NetworkAudit),
            _ => Err("Unknown edition"),
        }
    }
}

const KEYS_SIZE: i32 = KEY_CHARS.len() as i32;
const KEY_CHARS: [u8; 34] = [
    b'D', b'Y', b'1', b'4', b'U', b'F', b'3', b'R', b'H',
    b'W', b'C', b'X', b'L', b'Q', b'B', b'6', b'I', b'K',
    b'J', b'T', b'9', b'N', b'5', b'A', b'G', b'S', b'2',
    b'P', b'M', b'8', b'V', b'Z', b'7', b'E'
];

fn get_checksum<T: AsRef<[u8]>>(key_part: T) -> u16 {
    (key_part
        .as_ref()
        .iter()
        .fold(0u32, |result, b| {
            (0..8).fold(result ^ (*b as u32) << 8, |result, _| {
                if result & 0x8000 == 0 {
                    result << 1
                } else {
                    result << 1 ^ 0x8201
                }
            })
    }) & 0xFFFF) as u16
}

fn dec_part<T: AsRef<[u8]>>(key_part: T) -> i32 {
    key_part
        .as_ref()
        .iter()
        .enumerate()
        .fold(0i32, |result, (_, c1)| {
            (result * 34) + KEY_CHARS
                .iter()
                .position(|&c2| c2 == *c1)
                .unwrap_or(0) as i32
        })
}

fn enc_part(mut val: i32, slice: &mut [u8]) {
    slice
        .iter_mut()
        .rev()
        .for_each(|x| {
            *x = KEY_CHARS[(val % KEYS_SIZE) as usize];
            val /= KEYS_SIZE;
        })
}

fn gen_pair(slice: &mut [u8]) {
    slice
        .iter_mut()
        .for_each(|x| *x = KEY_CHARS[thread_rng().gen_range(0, KEYS_SIZE) as usize])
}

fn verify_checksum<T: AsRef<[u8]>>(key: T) -> bool {
    let key = key.as_ref();
    key.len() == 25 && {
        let mut enc_checksum: [u8; 3] = [0; 3];
        enc_part(get_checksum(&key[0..24]) as i32, &mut enc_checksum);
        enc_checksum[1] == key[24]
    }
}

fn is_valid_key<T: AsRef<[u8]>>(key: T) -> bool {
    let key = key.as_ref();
    verify_checksum(key) && {
        let key_parts: [i32; 9] = [
            dec_part(&key[22..24]),
            dec_part(&key[0..2]),
            dec_part(&key[2..4]),
            dec_part(&key[4..6]),
            dec_part(&key[6..8]),
            dec_part(&key[8..12]),
            dec_part(&key[12..16]),
            dec_part(&key[16..19]),
            dec_part(&key[19..22]),
        ];

        let edition = KeyEdition::try_from(((key_parts[0] & 0xFF) ^ key_parts[1] ^ 0xBF) - 1);
        let unk1 = (key_parts[0] & 0xFF) ^ key_parts[2] ^ 0xED;
        let unk2 = (key_parts[0] & 0xFF) ^ (key_parts[3] & 0xFFFF) ^ 0x77;
        let unk3 = (key_parts[0] & 0xFF) ^ (key_parts[4] & 0xFFFF) ^ 0xDF;
        let license_count = key_parts[0] ^ key_parts[5] ^ 0x4755;
        let purchase_date = key_parts[0] ^ key_parts[6] ^ 0x7CC1;
        let expire_val1 = (key_parts[0] & 0xFF) ^ key_parts[7] ^ 0x3FD;
        let expire_val2 = (key_parts[0] & 0xFF) ^ key_parts[7] ^ 0x935;

        edition.is_ok() &&
        unk1 < 990 && unk2 <= 100 &&  unk3 <= 100 &&
        license_count > 0 && license_count < 798 &&
        expire_val1 <= 3660 && expire_val2 > 0 && expire_val2 <= 3660 && (expire_val1 != 3660 || expire_val2 != 1830) &&
        purchase_date >= 577 && purchase_date <= 49567
    }
}

// example method for generating a key
fn generate_key(edition: KeyEdition, license_count: i32, purchase_val: i32, expire1_val: i32, expire2_val: i32) -> String {
    let mut rng = thread_rng();
    let unk1 = rng.gen_range(100, 989);
    let unk2 = rng.gen_range(0, 100) & 0xFFFF;
    let unk3 = rng.gen_range(0, 100) & 0xFFFF;

    let mut enc_key: [u8; 25] = [0; 25];
    gen_pair(&mut enc_key[22..24]);

    let base_val = dec_part(&mut enc_key[22..24]);
    enc_part((base_val & 0xFF) ^ (edition as i32 + 1) ^ 0xBF, &mut enc_key[0..2]);
    enc_part((base_val & 0xFF) ^ unk1 ^ 0xED, &mut enc_key[2..4]);
    enc_part((base_val & 0xFF) ^ unk2 ^ 0x77, &mut enc_key[4..6]);
    enc_part((base_val & 0xFF) ^ unk3 ^ 0xDF, &mut enc_key[6..8]);
    enc_part((base_val & 0xFFFFFF) ^ license_count ^ 0x4755, &mut enc_key[8..12]);
    enc_part((base_val & 0xFFFFFF) ^ purchase_val ^ 0x7CC1, &mut enc_key[12..16]);
    enc_part((base_val & 0xFF) ^ expire1_val ^ 0x3FD, &mut enc_key[16..19]);
    enc_part((base_val & 0xFF) ^ expire2_val ^ 0x935, &mut enc_key[19..22]);

    let mut enc_checksum: [u8; 3] = [0; 3];
    enc_part(get_checksum(&mut enc_key[0..24]) as i32, &mut enc_checksum);
    enc_key[24] = enc_checksum[1];

    String::from_utf8(enc_key.to_vec()).unwrap()
}

fn main() {
    let license_count = 1;
    let purchase_date = Date {day: 1, month: 2, year: 2020}.enc();
    let license_expire = 0;
    let maintance_expire = 3660;
    for i in 0..4 {
        let edition = KeyEdition::try_from(i).unwrap();
        let key = generate_key(edition, license_count, purchase_date, license_expire, maintance_expire);
        println!("generated key {} for edition {:?}", key, edition);
    }
}