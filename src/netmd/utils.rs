use crate::netmd::mappings::{ALLOWED_HW_KANA, MAPPINGS_DE, MAPPINGS_HW, MAPPINGS_JP, MAPPINGS_RU};
use diacritics;
use encoding_rs::SHIFT_JIS;
use regex::Regex;
use std::{collections::hash_map::HashMap, error::Error, vec::IntoIter};
use unicode_normalization::UnicodeNormalization;

extern crate kana;
use kana::*;

/// Sleep for a specified number of milliseconds on any platform
pub async fn cross_sleep(millis: u32) {
    #[cfg(not(target_family = "wasm"))]
    std::thread::sleep(std::time::Duration::from_millis(millis as u64));

    #[cfg(target_family = "wasm")]
    gloo::timers::future::TimeoutFuture::new(millis).await;
}

pub fn bcd_to_int(mut bcd: i32) -> i32 {
    let mut value = 0;
    let mut nibble = 0;

    while bcd != 0 {
        let nibble_value = bcd & 0xf;
        bcd >>= 4;
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

pub fn half_width_to_full_width_range(range: &str) -> String {
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

pub fn get_bytes<const S: usize>(iterator: &mut IntoIter<u8>) -> Result<[u8; S], Box<dyn Error>> {
    let byte_vec: Vec<u8> = iterator.take(S).collect();
    let bytes: [u8; S] = byte_vec.try_into().unwrap();

    Ok(bytes)
}

pub fn length_after_encoding_to_jis(string: &str) -> usize {
    let new_string = SHIFT_JIS.encode(string);

    new_string.0.len()
}

pub fn validate_shift_jis(sjis_string: Vec<u8>) -> bool {
    let (_, _, had_errors) = SHIFT_JIS.decode(&sjis_string);

    had_errors
}

fn check(string: String) -> Option<String> {
    if MAPPINGS_HW.contains_key(&string) {
        return Some(MAPPINGS_HW.get(&string).unwrap().to_string());
    }
    let mut ch = string.chars();
    if (ch.next().unwrap() as u32) < 0x7f || ALLOWED_HW_KANA.contains(&string) {
        return Some(string);
    }
    None
}

pub fn sanitize_half_width_title(title: &str) -> Vec<u8> {
    let mut string_title = wide2ascii(title);
    string_title = nowidespace(&string_title);
    string_title = hira2kata(&string_title);
    string_title = combine(&string_title);

    let new_title: String = string_title
        .chars()
        .map(|c| {
            check(c.to_string()).unwrap_or(
                check(diacritics::remove_diacritics(&c.to_string())).unwrap_or(" ".to_string()),
            )
        })
        .collect();

    let sjis_string = SHIFT_JIS.encode(&new_title).0;

    if validate_shift_jis(sjis_string.clone().into()) {
        return agressive_sanitize_title(title).into();
    }

    sjis_string.into()
}

// TODO: This function is bad, probably should do the string sanitization in the frontend
pub fn sanitize_full_width_title(title: &str, just_remap: bool) -> Vec<u8> {
    let new_title: String = title
        .chars()
        .map(|character| {
            match MAPPINGS_JP.get(&character.to_string()) {
                Some(string) => string.clone(),
                None => character.to_string().clone(),
            }
            .to_string()
        })
        .map(|character| {
            match MAPPINGS_RU.get(&character.to_string()) {
                Some(string) => string.clone(),
                None => character.to_string().clone(),
            }
            .to_string()
        })
        .map(|character| {
            match MAPPINGS_DE.get(&character.to_string()) {
                Some(string) => string.clone(),
                None => character.to_string().clone(),
            }
            .to_string()
        })
        .collect::<String>();

    if just_remap {
        return new_title.into();
    };

    let sjis_string = SHIFT_JIS.encode(&new_title).0;

    if validate_shift_jis(sjis_string.clone().into()) {
        return agressive_sanitize_title(title).into();
    }

    sjis_string.into()
}

pub fn agressive_sanitize_title(title: &str) -> String {
    let re = Regex::new(r"[^\x00-\x7F]").unwrap();
    re.replace_all(
        &diacritics::remove_diacritics(title)
            .nfd()
            .collect::<String>(),
        "",
    )
    .into()
}

pub fn time_to_duration(time: &Vec<u64>) -> std::time::Duration {
    assert_eq!(time.len(), 4);
    std::time::Duration::from_micros(
        (time[0] * 3600000000) + (time[1] * 60000000) + (time[2] * 1000000) + (time[3] * 11600),
    )
}
