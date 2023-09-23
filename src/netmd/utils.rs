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
        return Err("Upper nybble out of range [0..9]".into())
    }

    if lower >= 10 {
        return Err("Lower nybble out of range [0..9]".into())
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
