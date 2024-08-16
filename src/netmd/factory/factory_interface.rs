use std::time::Duration;

use log::debug;

use crate::netmd::{base::NetMDError, interface::{InterfaceError, NetmdStatus}, query_utils::{format_query, scan_query, QueryValue}, utils::{cross_sleep, to_sjis}, NetMD};


#[derive(Clone, Copy, Debug)]
pub enum MemoryType {
    Mapped = 0x0,
    Eeprom2 = 0x2,
    Eeprom3 = 0x3,
}

#[derive(Clone, Copy, Debug)]
pub enum MemoryOpenType {
    Close = 0x0,
    Read = 0x1,
    Write = 0x2,
    ReadWrite = 0x3,
}

#[derive(Clone, Copy, Debug)]
pub enum DisplayMode {
    Default = 0x0,
    Override = 0x1,
}

fn calculate_checksum(data: &[u8], seed: u32) -> u32 {
    let mut crc = seed;

    let mut tmp = data.len();
    data.iter().for_each(|e| {
        tmp = (tmp & 0xffff0000) | *e as usize;

        crc ^= tmp as u32;
        for i in 0..16 {
            let ts = crc & 0x8000;
            crc <<= i;
            if ts >= 1 {
                crc ^= 0x1021;
            }
        }
    });

    crc = (crc & 0xffff) >> 0;

    crc
}

fn calculate_eeprom_checksum(data: &[u8], seed: u32) -> u32 {
    let mut crc = seed;

    let mut tmp = data.len();

    // Convert the input into its little endian 16 bit representation
    let new_data: Vec<u16> = data
        .clone()
        .chunks(2)
        .map(|c| u16::from_le_bytes(c.try_into().unwrap()))
        .collect();

    new_data.iter().for_each(|e| {
        tmp = (tmp & 0xffff0000) | *e as usize;

        crc ^= tmp as u32;
        for i in 0..16 {
            let ts = crc & 0x8000;
            crc <<= i;
            if ts >= 1 {
                crc ^= 0x1021;
            }
        }
    });

    crc = (crc & 0xffff) >> 0;

    crc
}

pub struct NetMDFactoryInterface<'a> {
    device: &'a mut NetMD,
}

impl<'a> NetMDFactoryInterface<'a> {
    /// The maximum number of times to retry after an interim response
    const MAX_INTERIM_READ_ATTEMPTS: u8 = 4;

    /// The amount of time to wait after an interim response (in milliseconds)
    const INTERIM_RESPONSE_RETRY_INTERVAL: u32 = 100;

    pub fn new(device: &'a mut NetMD) -> NetMDFactoryInterface<'_> {
        Self { device }
    }

    pub async fn send_query(&mut self, query: &[u8], test: bool, accept_interim: bool) -> Result<Vec<u8>, InterfaceError> {
        self.send_command(query, test).await?;
        self.read_reply(accept_interim).await
    }

    pub async fn send_command(&mut self, query: &[u8], test: bool) -> Result<(), NetMDError> {
        let mut final_buffer = query.to_vec();
        if test {
            final_buffer.insert(0, NetmdStatus::SpecificInquiry as u8);
        } else {
            final_buffer.insert(0, NetmdStatus::Control as u8)
        }

        self.device.send_factory_command(final_buffer).await
    }

    pub async fn read_reply(&mut self, accept_interim: bool) -> Result<Vec<u8>, InterfaceError> {
        let mut current_attempt = 0;
        let mut data;

        while current_attempt < Self::MAX_INTERIM_READ_ATTEMPTS {
            data = self.device.read_factory_reply(None).await?;

            let status = NetmdStatus::try_from(data[0])?;
            debug!("Device status: {:?}", status);

            match status {
                NetmdStatus::NotImplemented => {
                    return Err(InterfaceError::NotImplemented(format!("{:02X?}", data)))
                }
                NetmdStatus::Rejected => {
                    return Err(InterfaceError::Rejected(format!("{:02X?}", data)))
                }
                NetmdStatus::Interim if !accept_interim => {
                    let sleep_time = Self::INTERIM_RESPONSE_RETRY_INTERVAL
                        * (u32::pow(2, current_attempt as u32) - 1);

                    cross_sleep(Duration::from_millis(sleep_time as u64)).await;

                    current_attempt += 1;
                    continue; // Retry!
                }
                NetmdStatus::Accepted | NetmdStatus::Implemented | NetmdStatus::Interim => {
                    if current_attempt >= Self::MAX_INTERIM_READ_ATTEMPTS {
                        return Err(InterfaceError::MaxRetries);
                    }
                    return Ok(data);
                }
                _ => return Err(InterfaceError::Unknown(format!("{:02X?}", data))),
            }
        }

        // This should NEVER happen unless the code is changed wrongly
        unreachable!("The max number of retries is set to 0!")
    }

    pub async fn auth(&mut self) -> Result<(), InterfaceError> {
        let query = format_query("1801 ff0e 4e6574204d442057616c6b6d616e", vec![])?;

        self.send_query(&query, false, false).await?;

        Ok(())
    }

    pub async fn change_memory_state(
        &mut self,
        address: u32,
        length: u32,
        mem_type: MemoryType,
        state: MemoryOpenType,
        encrypted: bool
    ) -> Result<(), InterfaceError> {
        let encrypted = match encrypted {
            true => 0x1,
            false => 0x0,
        };
        let query = format_query(
            "1820 ff %b %<d %b %b %b",
            vec![
                QueryValue::Number(mem_type as i64),
                QueryValue::Number(address as i64),
                QueryValue::Number(length as i64),
                QueryValue::Number(state as i64),
                QueryValue::Number(encrypted)
            ]
        )?;

        self.send_query(&query, false, false).await?;

        Ok(())
    }

    pub async fn read(
        &mut self,
        address: u32,
        length: u32,
        mem_type: MemoryType
    ) -> Result<Vec<u8>, InterfaceError> {
        let query = format_query(
            "1821 ff %b %<d %b",
            vec![
                QueryValue::Number(mem_type as i64),
                QueryValue::Number(address as i64),
                QueryValue::Number(length as i64),
            ]
        )?;

        let reply = self.send_query(&query, false, false).await?;
        let res = scan_query(reply, "1821 00 %? %?%?%?%? %? %?%? %*")?;
        let mut buf = res[0].to_vec().unwrap();
        buf = buf[..buf.len() - 2].to_vec();

        Ok(buf)
    }

    pub async fn write(
        &mut self,
        address: u32,
        data: &[u8],
        mem_type: MemoryType,
    ) -> Result<(), InterfaceError> {
        let checksum = calculate_checksum(data, 0);

        let query = format_query(
            "1822 ff %b %<d %b 0000 %* %<w",
            vec![
                QueryValue::Number(mem_type as i64),
                QueryValue::Number(address as i64),
                QueryValue::Number(data.len() as i64),
                QueryValue::Array(data.to_vec()),
                QueryValue::Number(checksum as i64),
            ]
        )?;

        self.send_query(&query, false, false).await?;

        Ok(())
    }

    pub async fn read_metadata_peripheral(
        &mut self,
        sector: u32,
        offset: u32,
        length: u32,
    ) -> Result<Vec<u8>, InterfaceError> {
        let query = format_query(
            "1824 ff %<w %<w %b",
            vec![
                QueryValue::Number(sector as i64),
                QueryValue::Number(offset as i64),
                QueryValue::Number(length as i64),
            ]
        )?;

        let reply = self.send_query(&query, false, false).await?;
        let res = scan_query(reply, "1821 00 %? %?%?%?%? %? %?%? %*")?;

        Ok(res[1].to_vec().unwrap())
    }

    pub async fn write_metadata_peripheral(
        &mut self,
        sector: u32,
        offset: u32,
        data: &[u8],
    ) -> Result<(), InterfaceError> {
        let query = format_query(
            "1824 ff %<w %<w %b",
            vec![
                QueryValue::Number(sector as i64),
                QueryValue::Number(offset as i64),
                QueryValue::Array(data.to_vec()),
            ]
        )?;

        self.send_query(&query, false, false).await?;
        Ok(())
    }

    pub async fn set_display_mode(
        &mut self,
        mode: DisplayMode,
    ) -> Result<(), InterfaceError> {
        let query = format_query(
            "1851 ff %b",
            vec![
                QueryValue::Number(mode as i64),
            ]
        )?;

        self.send_query(&query, false, false).await?;
        Ok(())
    }

    pub async fn set_display_override_string(
        &mut self,
        text: String,
        blink: bool,
    ) -> Result<(), InterfaceError> {
        assert!(!text.len() < 9);

        let blink = match blink {
            true => 0x1,
            false => 0x0,
        };

        let mut sjis_version = to_sjis(&text);
        sjis_version.extend_from_slice(&vec![0u8; 10 - sjis_version.len()]);

        let query = format_query(
            "1852 ff %b %b 00 %*",
            vec![
                QueryValue::Number(0),
                QueryValue::Number(blink),
                QueryValue::Array(sjis_version),
            ]
        )?;

        self.send_query(&query, false, false).await?;

        Ok(())
    }

    pub async fn get_device_version(&mut self) -> Result<u32, InterfaceError> {
        let query = format_query("1813 ff", vec![])?;
        let reply = self.send_query(&query, false, false).await?;
        let res = scan_query(reply, "1813 00 00 %B")?;

        Ok(res[0].to_i64().unwrap().try_into().unwrap())
    }

    pub async fn get_device_code(&mut self) -> Result<(u8, u8, u8, u8), InterfaceError> {
        let query = format_query("1812 ff", vec![])?;
        let reply = self.send_query(&query, false, false).await?;
        let res = scan_query(reply, "1812 00 %b %b %b %B")?;

        let chip_type: u8 = res[0].to_i64().unwrap().try_into().unwrap();
        let hwid: u8 = res[1].to_i64().unwrap().try_into().unwrap();
        let subversion: u8 = res[2].to_i64().unwrap().try_into().unwrap();
        let version: u8 = res[3].to_i64().unwrap().try_into().unwrap();

        Ok((
            chip_type,
            hwid,
            subversion,
            version,
        ))
    }

    pub async fn get_switch_status(&mut self) -> Result<(u16, u8, u8, u16), InterfaceError> {
        let query = format_query("1853 ff", vec![])?;
        let reply = self.send_query(&query, false, false).await?;
        let res = scan_query(reply, "1853 ff %w %b %b %w")?;

        let internal_microswitch: u16 = res[0].to_i64().unwrap().try_into().unwrap();
        let button: u8 = res[1].to_i64().unwrap().try_into().unwrap();
        let xy: u8 = res[2].to_i64().unwrap().try_into().unwrap();
        let unlabeled: u16 = res[3].to_i64().unwrap().try_into().unwrap();

        Ok((
            internal_microswitch,
            button,
            xy,
            unlabeled,
        ))
    }
}


