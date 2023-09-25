use std::collections::hash_map::HashMap;
use std::error::Error;

pub fn bcd_to_int(bcd: i32) -> i32 {
    let mut original = bcd;
    let mut value = 0;
    let mut nibble = 0;
    while original != 0 {
        let nibble_value = original & 0xf;
        original = original >> 4;
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

pub fn int_from_bcd(byte: u8) -> Result<u8, Box<dyn Error>> {
    let upper = (byte & 0xF0) >> 4;
    let lower = byte & 0x0F;

    if upper >= 10 {
        return Err("Upper nybble out of range [0..9]".into());
    }

    if lower >= 10 {
        return Err("Lower nybble out of range [0..9]".into());
    }

    Ok(upper * 10 + lower)
}

pub fn bcd_from_int(byte: u8) -> Result<u8, Box<dyn Error>> {
    let mut new_byte: u8 = 0;

    let upper = (byte / 10) << 4;
    let lower = byte % 10;

    new_byte |= upper;
    new_byte |= lower;

    Ok(new_byte)
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
