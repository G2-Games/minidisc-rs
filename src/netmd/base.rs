use nofmt;
use once_cell::sync::Lazy;
use rusb::{DeviceDescriptor, DeviceHandle, Direction, GlobalContext, Recipient, RequestType};
use std::error::Error;

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::new(1, 0);

const STANDARD_SEND: u8 =
    rusb::request_type(Direction::Out, RequestType::Vendor, Recipient::Interface);
const STANDARD_RECV: u8 =
    rusb::request_type(Direction::In, RequestType::Vendor, Recipient::Interface);

// TODO: I think this sucks, figure out a better way
pub static DEVICE_IDS: Lazy<Vec<DeviceId>> = Lazy::new(|| nofmt::pls!{
    Vec::from([
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
});

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

pub struct DeviceId {
    vendor_id: u16,
    product_id: u16,
    name: Option<String>,
}

pub struct NetMD {
    device_connection: DeviceHandle<GlobalContext>,
    model: DeviceId,
    status: Option<Status>,
}

impl NetMD {
    /// Creates a new `NetMD` struct
    pub fn new(
        device: DeviceHandle<GlobalContext>,
        device_desc: DeviceDescriptor,
    ) -> Result<Self, Box<dyn Error>> {
        let mut model = DeviceId {
            vendor_id: device_desc.vendor_id(),
            product_id: device_desc.product_id(),
            name: None,
        };

        for device_type in DEVICE_IDS.iter() {
            if device_type.vendor_id == model.vendor_id
                && device_type.product_id == model.product_id
            {
                model.name = device_type.name.clone()
            }
        }

        match model.name {
            None => return Err("Could not find device in list".into()),
            Some(_) => (),
        }

        Ok(Self {
            device_connection: device,
            model,
            status: None,
        })
    }

    /// Gets the device name from the struct
    pub fn device_name(&self) -> Option<String> {
        self.model.name.clone()
    }

    /// Gets the vendor id from the struct
    pub fn vendor_id(&self) -> u16 {
        self.model.vendor_id.clone()
    }

    /// Gets the product id from the struct
    pub fn product_id(&self) -> u16 {
        self.model.product_id.clone()
    }

    /// Poll the device to get either the result
    /// of the previous command, or the status
    fn poll(&self, tries: usize) -> Result<(usize, [u8; 4]), Box<dyn Error>> {
        // Create an array to store the result of the poll
        let mut poll_result = [0u8; 4];

        // Try until failure or `tries` reached
        for i in 0..tries {
            let _status = match self.device_connection.read_control(
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

            if poll_result[0] != 0 {
                return Ok((poll_result[2] as usize, poll_result));
            }

            if i > 0 {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }

        Ok((poll_result[2] as usize, poll_result))
    }

    /// Send a control message to the device
    pub fn send_command(
        &self,
        command: Vec<u8>,
        use_factory_command: bool,
    ) -> Result<(), Box<dyn Error>> {
        //First poll to ensure the device is ready
        match self.poll(1) {
            Ok(buffer) => match buffer.1[2] {
                0 => 0,
                _ => return Err("Device not ready!".into()),
            },
            Err(error) => return Err(error),
        };

        let _ = match use_factory_command {
            false => 0x80,
            true => 0xff,
        };

        match self.device_connection.write_control(
            STANDARD_SEND,
            0x80,
            0,
            0,
            &command,
            DEFAULT_TIMEOUT,
        ) {
            Ok(_) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    /// Poll to see if a message is ready,
    /// and if so, recieve it
    pub fn read_reply(&self, use_factory_command: bool) -> Result<Vec<u8>, Box<dyn Error>> {
        let poll_result = match self.poll(30) {
            Ok(buffer) => buffer,
            Err(error) => return Err(error),
        };

        let request = match use_factory_command {
            false => 0x81,
            true => 0xff,
        };

        // Create a buffer to fill with the result
        let mut buf: [u8; 255] = [0; 255];

        match self.device_connection.read_control(
            STANDARD_RECV,
            request,
            0,
            0,
            &mut buf,
            DEFAULT_TIMEOUT,
        ) {
            Ok(_) => Ok(buf[0..poll_result.0].to_vec()),
            Err(error) => return Err(error.into()),
        }
    }

    // TODO: Implement these properly, they will NOT work as is
    pub fn read_bulk<const S: usize>(&self, chunksize: u32) -> Result<Vec<u8>, Box<dyn Error>> {
        let result = self.read_bulk_to_array::<S>(chunksize)?;

        Ok(result.to_vec())
    }

    pub fn read_bulk_to_array<const S: usize>(&self, chunksize: u32) -> Result<[u8; S], Box<dyn Error>> {
        let mut buffer: [u8; S] = [0u8; S];

        self.device_connection.read_bulk(
            1,
            &mut buffer,
            DEFAULT_TIMEOUT
        )?;

        Ok(buffer)
    }
}
