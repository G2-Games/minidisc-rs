use std::collections::hash_map::HashMap;
use encoding_rs::SHIFT_JIS;
use std::error::Error;

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

pub fn length_after_encoding_to_jis(string: String) -> usize {
    let new_string = SHIFT_JIS.encode(&string);

    new_string.0.len()
}
