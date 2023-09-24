use std::collections::hash_map::HashMap;
use std::error::Error;
use once_cell::sync::Lazy;

// prettier-ignore
const FORMAT_TYPE_LEN_DICT: Lazy<HashMap<char, i32>> = Lazy::new(|| {
    HashMap::from([
        ('b', 1), // byte
        ('w', 2), // word
        ('d', 4), // doubleword
        ('q', 8), // quadword
    ])
});

/*
    %b, w, d, q - explained above (can have endiannes overriden by '>' and '<' operators, f. ex. %>d %<q)
    %s - Uint8Array preceded by 2 bytes of length
    %x - Uint8Array preceded by 2 bytes of length
    %z - Uint8Array preceded by 1 byte of length
    %* - raw Uint8Array
    %B - BCD-encoded 1-byte number
    %W - BCD-encoded 2-byte number
*/

const DEBUG: bool = false;

/// Formats a query using a standard input to send to the player
pub fn format_query(format: String, args: Vec<i32>) -> Result<Vec<u8>, Box<dyn Error>> {
    if DEBUG {
        println!("SENT>>> F: {}", format);
    }

    let mut result: Vec<u8> = Vec::new();

    for character in format.into_bytes().into_iter() {
        println!("{}", character);
    }

    Ok(Vec::new())
}
