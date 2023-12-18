use nofmt;
use once_cell::sync::Lazy;
use std::error::Error;
use yusb::{Device, DeviceHandle, Direction, Recipient, RequestType};

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::new(9999999, 0);

const STANDARD_SEND: u8 =
    yusb::request_type(Direction::Out, RequestType::Vendor, Recipient::Interface);
const STANDARD_RECV: u8 =
    yusb::request_type(Direction::In, RequestType::Vendor, Recipient::Interface);
const BULK_WRITE_ENDPOINT: u8 = 0x02;
const BULK_READ_ENDPOINT: u8 = 0x81;

pub static DEVICE_IDS: Lazy<Box<[DeviceId]>> = Lazy::new(|| {
    nofmt::pls! {
        Box::new([
            DeviceId {vendor_id: 0x04dd, product_id: 0x7202, name: Some(String::from("Sharp IM-MT899H"))},
            DeviceId {vendor_id: 0x04dd, product_id: 0x9013, name: Some(String::from("Sharp IM-DR400"))},
            DeviceId {vendor_id: 0x04dd, product_id: 0x9014, name: Some(String::from("Sharp IM-DR80"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0034, name: Some(String::from("Sony PCLK-XX"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0036, name: Some(String::from("Sony"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0075, name: Some(String::from("Sony MZ-N1"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x007c, name: Some(String::from("Sony"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0080, name: Some(String::from("Sony LAM-1"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0081, name: Some(String::from("Sony MDS-JB980/MDS-NT1/MDS-JE780"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0084, name: Some(String::from("Sony MZ-N505"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0085, name: Some(String::from("Sony MZ-S1"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0086, name: Some(String::from("Sony MZ-N707"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x008e, name: Some(String::from("Sony CMT-C7NT"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0097, name: Some(String::from("Sony PCGA-MDN1"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x00ad, name: Some(String::from("Sony CMT-L7HD"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x00c6, name: Some(String::from("Sony MZ-N10"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x00c7, name: Some(String::from("Sony MZ-N910"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x00c8, name: Some(String::from("Sony MZ-N710/NF810"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x00c9, name: Some(String::from("Sony MZ-N510/N610"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x00ca, name: Some(String::from("Sony MZ-NE410/NF520D"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x00e7, name: Some(String::from("Sony CMT-M333NT/M373NT"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x00eb, name: Some(String::from("Sony MZ-NE810/NE910"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0101, name: Some(String::from("Sony LAM"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0113, name: Some(String::from("Aiwa AM-NX1"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x013f, name: Some(String::from("Sony MDS-S500"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x014c, name: Some(String::from("Aiwa AM-NX9"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x017e, name: Some(String::from("Sony MZ-NH1"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0180, name: Some(String::from("Sony MZ-NH3D"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0182, name: Some(String::from("Sony MZ-NH900"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0184, name: Some(String::from("Sony MZ-NH700/NH800"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0186, name: Some(String::from("Sony MZ-NH600"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0187, name: Some(String::from("Sony MZ-NH600D"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0188, name: Some(String::from("Sony MZ-N920"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x018a, name: Some(String::from("Sony LAM-3"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x01e9, name: Some(String::from("Sony MZ-DH10P"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0219, name: Some(String::from("Sony MZ-RH10"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x021b, name: Some(String::from("Sony MZ-RH710/MZ-RH910"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x021d, name: Some(String::from("Sony CMT-AH10"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x022c, name: Some(String::from("Sony CMT-AH10"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x023c, name: Some(String::from("Sony DS-HMD1"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0286, name: Some(String::from("Sony MZ-RH1"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x011a, name: Some(String::from("Sony CMT-SE7"))},
            DeviceId {vendor_id: 0x054c, product_id: 0x0148, name: Some(String::from("Sony MDS-A1"))},
            DeviceId {vendor_id: 0x0b28, product_id: 0x1004, name: Some(String::from("Kenwood MDX-J9"))},
            DeviceId {vendor_id: 0x04da, product_id: 0x23b3, name: Some(String::from("Panasonic SJ-MR250"))},
            DeviceId {vendor_id: 0x04da, product_id: 0x23b6, name: Some(String::from("Panasonic SJ-MR270"))},
        ])
    }
});

/// The current status of the Minidisc device
pub enum Status {
    Ready,
    Playing,
    Paused,
    FastForward,
    Rewind,
    ReadingTOC,
    NoDisc,
    DiscBlank,
}

/// The ID of a device, including the name
pub struct DeviceId {
    vendor_id: u16,
    product_id: u16,
    name: Option<String>,
}

/// A connection to a NetMD device
pub struct NetMD {
    device: DeviceHandle,
    model: DeviceId,
    status: Option<Status>,
}

impl NetMD {
    const READ_REPLY_RETRY_INTERVAL: u32 = 10;

    /// Creates a new interface to a NetMD device
    pub fn new(device: Device) -> Result<Self, Box<dyn Error>> {
        let descriptor = device.device_descriptor()?;

        let mut model = DeviceId {
            vendor_id: descriptor.vendor_id(),
            product_id: descriptor.product_id(),
            name: None,
        };

        for device_type in DEVICE_IDS.iter() {
            if device_type.vendor_id == model.vendor_id
                && device_type.product_id == model.product_id
            {
                model.name = device_type.name.clone();
                break;
            }
        }

        match model.name {
            None => return Err("Could not find device in list".into()),
            Some(_) => (),
        }

        Ok(Self {
            device: device.open()?,
            model,
            status: None,
        })
    }

    /// Gets the device name, this is limited to the devices in the list
    pub fn device_name(&self) -> &Option<String> {
        &self.model.name
    }

    /// Gets the vendor id
    pub fn vendor_id(&self) -> &u16 {
        &self.model.vendor_id
    }

    /// Gets the product id
    pub fn product_id(&self) -> &u16 {
        &self.model.product_id
    }

    /// Poll the device to get either the result
    /// of the previous command, or the status
    pub fn poll(&mut self) -> Result<(u16, [u8; 4]), Box<dyn Error>> {
        // Create an array to store the result of the poll
        let mut poll_result = [0u8; 4];

        let _status = match self.device.read_control(
            STANDARD_RECV,
            0x01,
            0,
            0,
            &mut poll_result,
            DEFAULT_TIMEOUT,
        ) {
            Ok(size) => size,
            Err(error) => return Err(error.into()),
        };

        let length_bytes = [poll_result[2], poll_result[3]];
        Ok((u16::from_le_bytes(length_bytes), poll_result))
    }

    pub fn send_command(&mut self, command: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self._send_command(command, false)
    }

    pub fn send_factory_command(&mut self, command: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self._send_command(command, true)
    }

    /// Send a control message to the device
    fn _send_command(
        &mut self,
        command: Vec<u8>,
        use_factory_command: bool,
    ) -> Result<(), Box<dyn Error>> {
        // First poll to ensure the device is ready
        match self.poll() {
            Ok(buffer) => match buffer.1[2] {
                0 => 0,
                _ => return Err("Device not ready!".into()),
            },
            Err(error) => return Err(error),
        };

        let request = match use_factory_command {
            false => 0x80,
            true => 0xff,
        };

        match self
            .device
            .write_control(STANDARD_SEND, request, 0, 0, &command, DEFAULT_TIMEOUT)
        {
            Ok(_) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn read_reply(&mut self, override_length: Option<i32>) -> Result<Vec<u8>, Box<dyn Error>> {
        self._read_reply(false, override_length)
    }

    pub fn read_factory_reply(
        &mut self,
        override_length: Option<i32>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        self._read_reply(true, override_length)
    }

    /// Poll to see if a message is ready,
    /// and if so, recieve it
    fn _read_reply(
        &mut self,
        use_factory_command: bool,
        override_length: Option<i32>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut length = self.poll()?.0;

        let mut current_attempt = 0;
        while length == 0 {
            let sleep_time = Self::READ_REPLY_RETRY_INTERVAL as u64
                * (u64::pow(2, current_attempt as u32 / 10) - 1);

            std::thread::sleep(std::time::Duration::from_millis(sleep_time));
            length = self.poll()?.0;
            current_attempt += 1;
        }

        if let Some(value) = override_length {
            length = value as u16
        }

        let request = match use_factory_command {
            false => 0x81,
            true => 0xff,
        };

        // Create a buffer to fill with the result
        let mut buf: Vec<u8> = vec![0; length as usize];

        // Create a buffer to fill with the result
        match self
            .device
            .read_control(STANDARD_RECV, request, 0, 0, &mut buf, DEFAULT_TIMEOUT)
        {
            Ok(_) => Ok(buf),
            Err(error) => Err(error.into()),
        }
    }

    // Default chunksize should be 0x10000
    // TODO: Make these Async eventually
    pub fn read_bulk(&mut self, length: u32, chunksize: u32) -> Result<Vec<u8>, Box<dyn Error>> {
        let result = self.read_bulk_to_array(length, chunksize)?;

        Ok(result)
    }

    pub fn read_bulk_to_array(
        &mut self,
        length: u32,
        chunksize: u32,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut final_result: Vec<u8> = Vec::new();
        let mut done = 0;

        while done < length {
            let to_read = std::cmp::min(chunksize, length - done);
            done -= to_read;
            let mut buffer: Vec<u8> = vec![0; to_read as usize];

            match self
                .device
                .read_bulk(BULK_READ_ENDPOINT, &mut buffer, DEFAULT_TIMEOUT)
            {
                Ok(result) => result,
                Err(error) => return Err(format!("USB error: {:?}", error).into()),
            };

            final_result.extend_from_slice(&buffer);
        }

        Ok(final_result)
    }

    pub fn write_bulk(&mut self, data: &mut [u8]) -> Result<usize, Box<dyn Error>> {
        let written = self
            .device
            .write_bulk(BULK_WRITE_ENDPOINT, data, DEFAULT_TIMEOUT)?;

        Ok(written)
    }
}
