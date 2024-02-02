#![cfg_attr(debug_assertions, allow(dead_code))]
use nofmt;
use once_cell::sync::Lazy;
use std::error::Error;
use std::time::Duration;

// USB stuff
//use nusb::transfer::{Control, ControlIn, ControlOut, ControlType, Recipient, RequestBuffer};
use cross_usb::{UsbDevice, UsbInterface};
use cross_usb::usb::{ControlIn, ControlOut, ControlType, Device, Interface, Recipient};

use super::utils::cross_sleep;
//use nusb::{Device, DeviceInfo, Interface};

const DEFAULT_TIMEOUT: Duration = Duration::new(10000, 0);
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
    usb_interface: UsbInterface,
    model: DeviceId,
    status: Option<Status>,
}

impl NetMD {
    const READ_REPLY_RETRY_INTERVAL: u32 = 10;

    /// Creates a new interface to a NetMD device
    pub async fn new(usb_device: &UsbDevice) -> Result<Self, Box<dyn Error>> {
        let mut model = DeviceId {
            vendor_id: usb_device.vendor_id().await,
            product_id: usb_device.product_id().await,
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

        let usb_interface = usb_device.open_interface(0).await?;

        Ok(Self {
            usb_interface,
            model,
            status: None,
        })
    }

    /// Gets the device name, this is limited to the devices in the list
    pub async fn device_name(&self) -> &Option<String> {
        &self.model.name
    }

    /// Gets the vendor id
    pub async fn vendor_id(&self) -> &u16 {
        &self.model.vendor_id
    }

    /// Gets the product id
    pub async fn product_id(&self) -> &u16 {
        &self.model.product_id
    }

    /// Poll the device to get either the result
    /// of the previous command, or the status
    pub async fn poll(&mut self) -> Result<(u16, [u8; 4]), Box<dyn Error>> {
        // Create an array to store the result of the poll
        let poll_result = match self
            .usb_interface
            .control_in(ControlIn {
                control_type: ControlType::Vendor,
                recipient: Recipient::Interface,
                request: 0x01,
                value: 0,
                index: 0,
                length: 4,
            })
            .await
        {
            Ok(size) => size,
            Err(error) => return Err(error.into()),
        };

        let length_bytes = u16::from_le_bytes([poll_result[2], poll_result[3]]);

        let poll_result: [u8; 4] = match poll_result.try_into() {
            Ok(val) => val,
            Err(_) => return Err("could not convert result".into()),
        };

        Ok((length_bytes, poll_result))
    }

    pub async fn send_command(&mut self, command: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self._send_command(command, false).await
    }

    pub async fn send_factory_command(&mut self, command: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self._send_command(command, true).await
    }

    /// Send a control message to the device
    async fn _send_command(
        &mut self,
        command: Vec<u8>,
        use_factory_command: bool,
    ) -> Result<(), Box<dyn Error>> {
        // First poll to ensure the device is ready
        match self.poll().await {
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
            .usb_interface
            .control_out(ControlOut {
                control_type: ControlType::Vendor,
                recipient: Recipient::Interface,
                request,
                value: 0,
                index: 0,
                data: &command,
            })
            .await
        {
            Ok(_) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    pub async fn read_reply(
        &mut self,
        override_length: Option<i32>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        self._read_reply(false, override_length).await
    }

    pub async fn read_factory_reply(
        &mut self,
        override_length: Option<i32>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        self._read_reply(true, override_length).await
    }

    /// Poll to see if a message is ready, and once it is, retrieve it
    async fn _read_reply(
        &mut self,
        use_factory_command: bool,
        override_length: Option<i32>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut length = 0;

        for attempt in 0..40 {
            if attempt == 39 {
                return Err("Failed to get response length".into());
            }

            length = self.poll().await?.0;

            if length > 0 {
                break
            }

            // Back off while trying again
            let sleep_time = Self::READ_REPLY_RETRY_INTERVAL
                * (u32::pow(2, attempt / 10) - 1);

            cross_sleep(sleep_time).await;
        }

        if let Some(value) = override_length {
            length = value as u16
        }

        let request = match use_factory_command {
            false => 0x81,
            true => 0xff,
        };

        // Create a buffer to fill with the result
        let reply = self
            .usb_interface
            .control_in(ControlIn {
                control_type: ControlType::Vendor,
                recipient: Recipient::Interface,
                request,
                value: 0,
                index: 0,
                length,
            })
            .await?;

        Ok(reply)
    }

    // Default chunksize should be 0x10000
    pub async fn read_bulk(
        &mut self,
        length: usize,
        chunksize: usize,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let result = self.read_bulk_to_array(length, chunksize).await?;

        Ok(result)
    }

    pub async fn read_bulk_to_array(
        &mut self,
        length: usize,
        chunksize: usize,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut final_result: Vec<u8> = Vec::new();
        let mut done = 0;

        while done < length {
            let to_read = std::cmp::min(chunksize, length - done);
            done -= to_read;

            let res = match self
                .usb_interface
                .bulk_in(BULK_READ_ENDPOINT, to_read)
                .await
            {
                Ok(result) => result,
                Err(error) => return Err(format!("USB error: {:?}", error).into()),
            };

            final_result.extend_from_slice(&res);
        }

        Ok(final_result)
    }

    pub async fn write_bulk(&mut self, data: &[u8]) -> Result<usize, Box<dyn Error>> {
        Ok(self.usb_interface.bulk_out(BULK_WRITE_ENDPOINT, data).await?)
    }
}
