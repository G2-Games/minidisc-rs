use std::error::Error;

pub fn check_result(result: Vec<u8>, expected: &[u8]) -> Result<(), Box<dyn Error>> {
    match result.as_slice().eq(expected) {
        true => Ok(()),
        false => Err("Response was not expected!".into()),
    }
}
