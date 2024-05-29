use crate::netmd::mappings::{ALLOWED_HW_KANA, MAPPINGS_DE, MAPPINGS_HW, MAPPINGS_JP, MAPPINGS_RU};
use diacritics;
use encoding_rs::SHIFT_JIS;
use regex::Regex;
use std::{error::Error, io::Write, time::Duration, vec::IntoIter};
use unicode_normalization::UnicodeNormalization;
use byteorder::{LittleEndian, WriteBytesExt};

extern crate kana;
use kana::*;

use super::{interface::DiscFormat, mappings::{HW_TO_FW_RANGE_MAP, MULTI_BYTE_CHARS}};

/// Sleep for a specified [Duration] on any platform
pub async fn cross_sleep(duration: Duration) {
    #[cfg(not(target_family = "wasm"))]
    std::thread::sleep(duration);

    #[cfg(target_family = "wasm")]
    gloo::timers::future::TimeoutFuture::new(duration.as_millis() as u32).await;
}

pub fn bcd_to_int(mut bcd: i32) -> i32 {
    let mut value = 0;
    let mut nybble = 0;

    while bcd != 0 {
        let nybble_value = bcd & 0xf;
        bcd >>= 4;
        value += nybble_value * i32::pow(10, nybble);
        nybble += 1;
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
    range
        .chars()
        .map(|char| HW_TO_FW_RANGE_MAP.get(&char).unwrap())
        .collect()
}

pub fn get_bytes<const S: usize>(iterator: &mut IntoIter<u8>) -> Result<[u8; S], Box<dyn Error>> {
    let byte_vec: Vec<u8> = iterator.take(S).collect();
    let bytes: [u8; S] = byte_vec.try_into().unwrap();

    Ok(bytes)
}

pub fn length_after_encoding_to_sjis(string: &str) -> usize {
    let new_string = SHIFT_JIS.encode(string);

    new_string.0.len()
}

pub fn validate_sjis(sjis_string: Vec<u8>) -> bool {
    let (_, _, had_errors) = SHIFT_JIS.decode(&sjis_string);

    had_errors
}

/// Ensure string contains only hardware allowed characters
fn check(string: String) -> Option<String> {
    if MAPPINGS_HW.contains_key(string.as_str()) {
        return Some(MAPPINGS_HW.get(string.as_str()).unwrap().to_string());
    }
    let mut ch = string.chars();
    if (ch.next().unwrap() as u32) < 0x7f || ALLOWED_HW_KANA.contains(&string.as_str()) {
        return Some(string);
    }
    None
}

pub fn half_width_title_length(title: &str) -> usize {
    let multibyte_len = title.chars()
        .map(|c| (*MULTI_BYTE_CHARS.get(&c).unwrap_or(&0) as usize))
        .reduce(|a, b| a + b).unwrap_or_default();

    title.len() + multibyte_len
}

pub fn sanitize_half_width_title(title: &str) -> String {
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

    new_title
}

// TODO: This function is bad, probably should do the string sanitization in the frontend
pub fn sanitize_full_width_title(title: &str) -> String {
    let new_title: String = title
        .chars()
        .map(|c| c.to_string())
        .map(|character| {
            match MAPPINGS_JP.get(character.to_string().as_str()) {
                Some(string) => string,
                None => character.as_str(),
            }
            .to_string()
        })
        .map(|character| {
            match MAPPINGS_RU.get(character.as_str()) {
                Some(string) => string,
                None => character.as_str(),
            }
            .to_string()
        })
        .map(|character| {
            match MAPPINGS_DE.get(character.as_str()) {
                Some(string) => string,
                None => character.as_str(),
            }
            .to_string()
        })
        .collect::<String>();

    new_title
}

/// Convert a UTF-8 string to Shift-JIS for use on the player
pub fn to_sjis(sjis_str: &str) -> Vec<u8> {
    let sjis_string = SHIFT_JIS.encode(&sjis_str).0;

    if validate_sjis(sjis_string.clone().into()) {
        return agressive_sanitize_title(sjis_str).into();
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

pub struct AeaOptions<'a> {
    pub name: &'a str,
    pub channels: u32,
    pub sound_groups: u32,
    pub group_start: u32,
    pub encrypted: u32,
    pub flags: &'a [u8],
}

impl <'a> Default for AeaOptions<'a> {
    fn default() -> Self {
        Self {
            name: "",
            channels: 2,
            sound_groups: 1,
            group_start: 0,
            encrypted: 0,
            flags: &[0, 0, 0, 0, 0, 0, 0, 0]
        }
    }
}

pub fn create_aea_header(options: AeaOptions) -> Vec<u8> {
    let encoded_name = options.name.as_bytes();

    let mut header: Vec<u8> = Vec::new();

    header.write_u32::<LittleEndian>(2048).unwrap();
    header.write_all(encoded_name).unwrap();
    header.write_all(&vec![0; 256 - encoded_name.len()]).unwrap();
    header.write_u32::<LittleEndian>(options.sound_groups as u32).unwrap();
    header.write_all(&[options.channels as u8, 0]).unwrap();

    // Write the flags
    header.write_u32::<LittleEndian>(options.flags[0] as u32).unwrap();
    header.write_u32::<LittleEndian>(options.flags[1] as u32).unwrap();
    header.write_u32::<LittleEndian>(options.flags[2] as u32).unwrap();
    header.write_u32::<LittleEndian>(options.flags[3] as u32).unwrap();
    header.write_u32::<LittleEndian>(options.flags[4] as u32).unwrap();
    header.write_u32::<LittleEndian>(options.flags[5] as u32).unwrap();
    header.write_u32::<LittleEndian>(options.flags[6] as u32).unwrap();
    header.write_u32::<LittleEndian>(options.flags[7] as u32).unwrap();

    header.write_u32::<LittleEndian>(0).unwrap();

    header.write_u32::<LittleEndian>(options.encrypted as u32).unwrap();
    header.write_u32::<LittleEndian>(options.group_start as u32).unwrap();

    // return the header
    header
}

pub fn create_wav_header(format: DiscFormat, bytes: u32) -> Vec<u8> {
    let mut header: Vec<u8> = Vec::new();

    let (joint_stereo, bytes_per_frame) = match format {
        DiscFormat::LP4 => (192, 0),
        DiscFormat::LP2 => (96, 1),
        _ => unreachable!("Cannot create WAV header for disc type {:?}", format)
    };

    let bytes_per_second = (bytes_per_frame * 44100) / 512;

    header.write_all(r"RIFF".as_bytes()).unwrap();
    header.write_u32::<LittleEndian>(bytes + 60).unwrap();
    header.write_all(r"WAVEfmt".as_bytes()).unwrap();
    header.write_u32::<LittleEndian>(32).unwrap();
    header.write_u16::<LittleEndian>(0x270).unwrap(); // ATRAC3
    header.write_u16::<LittleEndian>(2).unwrap(); // Stereo
    header.write_u32::<LittleEndian>(44100).unwrap();
    header.write_u32::<LittleEndian>(bytes_per_second).unwrap();
    header.write_u16::<LittleEndian>(bytes_per_frame as u16 * 2).unwrap();

    header.write_all(&[0, 0]).unwrap();

    header.write_u16::<LittleEndian>(14).unwrap();
    header.write_u16::<LittleEndian>(1).unwrap();
    header.write_u32::<LittleEndian>(bytes_per_frame).unwrap();
    header.write_u16::<LittleEndian>(joint_stereo).unwrap();
    header.write_u16::<LittleEndian>(joint_stereo).unwrap();

    header.write_u16::<LittleEndian>(1).unwrap();
    header.write_u16::<LittleEndian>(0).unwrap();

    header.write_all(r"data".as_bytes()).unwrap();

    header.write_u32::<LittleEndian>(bytes).unwrap();

    header
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawTime {
    pub hours: u64,
    pub minutes: u64,
    pub seconds: u64,
    pub frames: u64,
}

impl Into<Duration> for RawTime {
    fn into(self) -> std::time::Duration {
        self.as_duration()
    }
}

impl RawTime {
    pub fn as_duration(&self) -> Duration {
        std::time::Duration::from_micros(
            (self.hours * 3600000000) + (self.minutes * 60000000) + (self.seconds * 1000000) + (self.frames * 11600),
        )
    }

    pub fn as_frames(&self) -> u64 {
        ((self.hours * 60 + self.minutes) * 60 + self.seconds) * 512 + self.frames
    }
}
