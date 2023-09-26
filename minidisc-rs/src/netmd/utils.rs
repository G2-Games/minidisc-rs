use encoding_rs::SHIFT_JIS;
use kana::{ascii2wide, combine, half2kana};
use std::collections::hash_map::HashMap;
use std::error::Error;
use crate::netmd::mappings::{MAPPINGS_JP, MAPPINGS_RU, MAPPINGS_DE};

pub fn bcd_to_int(mut bcd: i32) -> i32 {
    let mut value = 0;
    let mut nibble = 0;

    while bcd != 0 {
        let nibble_value = bcd & 0xf;
        bcd = bcd >> 4;
        value += nibble_value * i32::pow(10, nibble);
        nibble += 1;
    }

    value
}

pub fn int_to_bcd(mut value: i32) -> i32 {
    let mut bcd = 0;
    let mut shift = 0;

    while value > 0 {
        let digit = value % 10;
        bcd |= digit << shift;
        shift += 4;
        value /= 10;
    }

    bcd
}

pub fn half_width_to_full_width_range(range: &String) -> String {
    let mappings: HashMap<char, char> = HashMap::from([
        ('0', '０'),
        ('1', '１'),
        ('2', '２'),
        ('3', '３'),
        ('4', '４'),
        ('5', '５'),
        ('6', '６'),
        ('7', '７'),
        ('8', '８'),
        ('9', '９'),
        ('-', '－'),
        ('/', '／'),
        (';', '；'),
    ]);

    range
        .chars()
        .map(|char| mappings.get(&char).unwrap())
        .collect()
}

pub fn get_bytes<const S: usize>(
    iterator: &mut std::vec::IntoIter<u8>,
) -> Result<[u8; S], Box<dyn std::error::Error>> {
    let mut bytes = [0; S];

    for i in 0..S {
        bytes[i] = match iterator.next() {
            Some(byte) => byte,
            None => return Err("Could not retrieve byte from file".into()),
        };
    }

    Ok(bytes)
}

pub fn length_after_encoding_to_jis(string: &String) -> usize {
    let new_string = SHIFT_JIS.encode(string);

    new_string.0.len()
}

pub fn validate_shift_jis(sjis_string: &Vec<u8>) -> Result<(), Box<dyn Error>> {
    let (_, _, had_errors) = SHIFT_JIS.decode(sjis_string);

    if had_errors {
        Err("Not valid SHIFT-JIS".into())
    } else {
        Ok(())
    }
}

// TODO: This function is bad, probably should do the string sanitization in the frontend
pub fn sanitize_full_width_title(title: &String, just_remap: bool) -> Vec<u8> {
    let new_title: String = title
        .chars()
        .map(|character| {
            match MAPPINGS_JP.get(&character.to_string()) {
                Some(string) => string.clone(),
                None => character.to_string().clone()
            }.to_string()
        })
        .map(|character| {
            match MAPPINGS_RU.get(&character.to_string()) {
                Some(string) => string.clone(),
                None => character.to_string().clone()
            }.to_string()
        })
        .map(|character| {
            match MAPPINGS_DE.get(&character.to_string()) {
                Some(string) => string.clone(),
                None => character.to_string().clone()
            }.to_string()
        })
        .collect::<String>();

    if just_remap {
        return new_title.into();
    };

    let sjis_string = SHIFT_JIS.encode(&new_title).0;

    return sjis_string.into();
}
