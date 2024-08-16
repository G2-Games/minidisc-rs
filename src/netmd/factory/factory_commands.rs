use crate::netmd::interface::InterfaceError;

use super::factory_interface::{DisplayMode, MemoryOpenType, MemoryType, NetMDFactoryInterface};

pub enum PatchPeripheralBase{
    NetMD   = 0x03802000,
    HiMD    = 0x03804000,
}

pub async fn display_string(
    interface: &mut NetMDFactoryInterface<'_>,
    text: String,
    blink: bool,
) -> Result<(), InterfaceError> {
    interface.set_display_mode(DisplayMode::Override).await?;
    interface.set_display_override_string(text, blink).await?;
    Ok(())
}

/*
pub async fn clean_read(
    &mut self,
    address: u32,
    length: u32,
    mem_type: MemoryType,
    encrypted: bool,
    auto_decrypt: bool,
) -> Result<(), InterfaceError> {
    self.factory_interface.change_memory_state(address, length, mem_type, MemoryOpenType::Read, encrypted).await?;
    let res = self.factory_interface.read(address, length, mem_type).await?;
    self.factory_interface.change_memory_state(address, length, mem_type, MemoryOpenType::Close, encrypted).await?;

    if encrypted && auto_decrypt {

    }

    Ok(())
}
*/

pub async fn get_descriptive_device_code(
    interface: &mut NetMDFactoryInterface<'_>,
) -> Result<String, InterfaceError> {
    let (
        chip_type,
            _hwid,
            version,
            subversion
    ) = interface.get_device_code().await?;

    let code = match chip_type {
        0x20 => "R", // Type-R
        0x21 => "S", // Type-S
        0x23 => "Hp", // DH10P
        0x22 => "Hn", // MZ-NH*
        0x24 => "Hr", // MZ-RH*
        0x25 => "Hx", // MZ-RH1
        _ => &format!("{}", chip_type),
    };
    let mut code = code.to_string();

    let version: Vec<char> = version.to_string().chars().collect();
    let (maj, min) = (version[0], version[1]);

    code.push_str(&format!("{maj}.{min}{:02X}", subversion));

    Ok(code)
}
