use std::collections::hash_map::HashMap;
use std::error::Error;

pub fn check_result(result: Vec<u8>, expected: &[u8]) -> Result<(), Box<dyn Error>> {
    match result.as_slice().eq(expected) {
        true => Ok(()),
        false => Err("Response was not as expected!".into()),
    }
}

pub fn byte_from_bcd(byte: u8) -> Result<u8, Box<dyn Error>> {
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

pub fn bcd_from_byte(byte: u8) -> Result<u8, Box<dyn Error>> {
    let mut new_byte: u8 = 0;

    let upper = (byte / 10) << 4;
    let lower = byte % 10;

    new_byte |= upper;
    new_byte |= lower;

    Ok(new_byte)
}

pub fn half_width_to_full_width_range(range: String) -> String {
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
