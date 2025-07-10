#![cfg_attr(debug_assertions, allow(dead_code))]
use std::sync::LazyLock;
use std::time::Duration;

use thiserror::Error;

// USB stuff
use cross_usb::prelude::*;
use cross_usb::usb::{ControlIn, ControlOut, ControlType, Recipient, Error};
use cross_usb::{DeviceInfo, Interface};

use super::utils::cross_sleep;

const BULK_WRITE_ENDPOINT: u8 = 0x02;
const BULK_READ_ENDPOINT: u8 = 0x81;

nofmt::pls! { // Skip formatting the following info
/// Device IDs for use in matching existing devices
pub static DEVICE_IDS: &[DeviceId] = &[
    DeviceId { vendor_id: 0x04dd, product_id: 0x7202, name: Some("Sharp IM-MT899H") },
    DeviceId { vendor_id: 0x04dd, product_id: 0x9013, name: Some("Sharp IM-DR400") },
    DeviceId { vendor_id: 0x04dd, product_id: 0x9014, name: Some("Sharp IM-DR80") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0034, name: Some("Sony PCLK-XX") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0036, name: Some("Sony") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0075, name: Some("Sony MZ-N1") },
    DeviceId { vendor_id: 0x054c, product_id: 0x007c, name: Some("Sony") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0080, name: Some("Sony LAM-1") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0081, name: Some("Sony MDS-JB980/MDS-NT1/MDS-JE780") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0084, name: Some("Sony MZ-N505") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0085, name: Some("Sony MZ-S1") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0086, name: Some("Sony MZ-N707") },
    DeviceId { vendor_id: 0x054c, product_id: 0x008e, name: Some("Sony CMT-C7NT") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0097, name: Some("Sony PCGA-MDN1") },
    DeviceId { vendor_id: 0x054c, product_id: 0x00ad, name: Some("Sony CMT-L7HD") },
    DeviceId { vendor_id: 0x054c, product_id: 0x00c6, name: Some("Sony MZ-N10") },
    DeviceId { vendor_id: 0x054c, product_id: 0x00c7, name: Some("Sony MZ-N910") },
    DeviceId { vendor_id: 0x054c, product_id: 0x00c8, name: Some("Sony MZ-N710/NF810") },
    DeviceId { vendor_id: 0x054c, product_id: 0x00c9, name: Some("Sony MZ-N510/N610") },
    DeviceId { vendor_id: 0x054c, product_id: 0x00ca, name: Some("Sony MZ-NE410/NF520D") },
    DeviceId { vendor_id: 0x054c, product_id: 0x00e7, name: Some("Sony CMT-M333NT/M373NT") },
    DeviceId { vendor_id: 0x054c, product_id: 0x00eb, name: Some("Sony MZ-NE810/NE910") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0101, name: Some("Sony LAM") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0113, name: Some("Aiwa AM-NX1") },
    DeviceId { vendor_id: 0x054c, product_id: 0x013f, name: Some("Sony MDS-S500") },
    DeviceId { vendor_id: 0x054c, product_id: 0x014c, name: Some("Aiwa AM-NX9") },
    DeviceId { vendor_id: 0x054c, product_id: 0x017e, name: Some("Sony MZ-NH1") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0180, name: Some("Sony MZ-NH3D") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0182, name: Some("Sony MZ-NH900") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0184, name: Some("Sony MZ-NH700/NH800") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0186, name: Some("Sony MZ-NH600") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0187, name: Some("Sony MZ-NH600D") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0188, name: Some("Sony MZ-N920") },
    DeviceId { vendor_id: 0x054c, product_id: 0x018a, name: Some("Sony LAM-3") },
    DeviceId { vendor_id: 0x054c, product_id: 0x01e9, name: Some("Sony MZ-DH10P") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0219, name: Some("Sony MZ-RH10") },
    DeviceId { vendor_id: 0x054c, product_id: 0x021b, name: Some("Sony MZ-RH710/MZ-RH910") },
    DeviceId { vendor_id: 0x054c, product_id: 0x021d, name: Some("Sony CMT-AH10") },
    DeviceId { vendor_id: 0x054c, product_id: 0x022c, name: Some("Sony CMT-AH10") },
    DeviceId { vendor_id: 0x054c, product_id: 0x023c, name: Some("Sony DS-HMD1") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0286, name: Some("Sony MZ-RH1") },
    DeviceId { vendor_id: 0x054c, product_id: 0x011a, name: Some("Sony CMT-SE7") },
    DeviceId { vendor_id: 0x054c, product_id: 0x0148, name: Some("Sony MDS-A1") },
    DeviceId { vendor_id: 0x0b28, product_id: 0x1004, name: Some("Kenwood MDX-J9") },
    DeviceId { vendor_id: 0x04da, product_id: 0x23b3, name: Some("Panasonic SJ-MR250") },
    DeviceId { vendor_id: 0x04da, product_id: 0x23b6, name: Some("Panasonic SJ-MR270") },
];
}

/// Device IDs for use with [cross_usb]
pub static DEVICE_IDS_CROSSUSB: LazyLock<Box<[cross_usb::DeviceFilter]>> = LazyLock::new(|| {
    DEVICE_IDS
        .iter()
        .map(|d| {
            cross_usb::device_filter! {
                vendor_id: d.vendor_id,
                product_id: d.product_id,
            }
        })
        .collect()
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceId {
    vendor_id: u16,
    product_id: u16,
    name: Option<&'static str>,
}

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NetMDError {
    #[error("communication timed out")]
    Timeout,

    #[error("invalid usb result")]
    InvalidResult,

    #[error("the device is not ready")]
    NotReady,

    #[error("could not find device")]
    UnknownDevice(DeviceId),

    #[error("usb connection error")]
    UsbError(#[from] Error),
}

/// A low-level USB connection to a NetMD device.
///
/// With this you can send raw commands to the device and recieve raw data.
///
/// For simple communication with a NetMD device, you most likely want the
/// higher level [`super::NetMDInterface`] or [`super::NetMDContext`] interfaces
pub struct NetMD {
    usb_interface: Interface,
    model: DeviceId,
}

impl NetMD {
    const READ_REPLY_RETRY_INTERVAL: u32 = 10;

    /// Creates a new interface to a NetMD device
    pub async fn new(usb_descriptor: DeviceInfo) -> Result<Self, NetMDError> {
        let mut model = DeviceId {
            vendor_id: usb_descriptor.vendor_id().await,
            product_id: usb_descriptor.product_id().await,
            name: None,
        };

        for device_type in DEVICE_IDS.iter() {
            if device_type.vendor_id == model.vendor_id
                && device_type.product_id == model.product_id
            {
                model.name = device_type.name;
                break;
            }
        }

        match model.name {
            None => return Err(NetMDError::UnknownDevice(model)),
            Some(_) => (),
        }

        let usb_device = usb_descriptor.open().await?;
        let usb_interface = usb_device.open_interface(0).await?;

        Ok(Self {
            usb_interface,
            model,
        })
    }

    /// Gets the device name, this is limited to the devices in the list
    pub fn device_name(&self) -> Option<&str> {
        self.model.name
    }

    /// Gets the vendor id
    pub fn vendor_id(&self) -> u16 {
        self.model.vendor_id
    }

    /// Gets the product id
    pub fn product_id(&self) -> u16 {
        self.model.product_id
    }

    /// Poll the device to get either the result
    /// of the previous command, or the status
    pub async fn poll(&mut self) -> Result<(u16, [u8; 4]), NetMDError> {
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
            Err(_) => return Err(NetMDError::InvalidResult),
        };

        Ok((length_bytes, poll_result))
    }

    /// Send a control message to the device (Raw bytes)
    pub async fn send_command(&mut self, command: Vec<u8>) -> Result<(), NetMDError> {
        self._send_command(command, false).await
    }

    /// Send a factory control message to the device (Raw bytes)
    pub async fn send_factory_command(&mut self, command: Vec<u8>) -> Result<(), NetMDError> {
        self._send_command(command, true).await
    }

    /// Send a control message to the device, can also send factory commands
    async fn _send_command(
        &mut self,
        command: Vec<u8>,
        use_factory_command: bool,
    ) -> Result<(), NetMDError> {
        // First poll to ensure the device is ready
        match self.poll().await {
            Ok(buffer) => match buffer.1[2] {
                0 => 0,
                _ => return Err(NetMDError::NotReady),
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
    ) -> Result<Vec<u8>, NetMDError> {
        self._read_reply(false, override_length).await
    }

    pub async fn read_factory_reply(
        &mut self,
        override_length: Option<i32>,
    ) -> Result<Vec<u8>, NetMDError> {
        self._read_reply(true, override_length).await
    }

    /// Poll to see if a message is ready, and once it is, retrieve it
    async fn _read_reply(
        &mut self,
        use_factory_command: bool,
        override_length: Option<i32>,
    ) -> Result<Vec<u8>, NetMDError> {
        let mut length = 0;

        for attempt in 0..40 {
            if attempt == 39 {
                return Err(NetMDError::Timeout);
            }

            length = self.poll().await?.0;

            if length > 0 {
                break;
            }

            // Back off while trying again
            let sleep_time = Self::READ_REPLY_RETRY_INTERVAL * (u32::pow(2, attempt) - 1);

            cross_sleep(Duration::from_millis(sleep_time as u64)).await;
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
    pub async fn read_bulk<F: Fn(usize, usize)>(
        &mut self,
        length: usize,
        chunksize: usize,
        progress_callback: Option<F>,
    ) -> Result<Vec<u8>, NetMDError> {
        let result = self
            .read_bulk_to_array(length, chunksize, progress_callback)
            .await?;

        Ok(result)
    }

    pub async fn read_bulk_to_array<F: Fn(usize, usize)>(
        &mut self,
        length: usize,
        chunksize: usize,
        progress_callback: Option<F>,
    ) -> Result<Vec<u8>, NetMDError> {
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
                Err(error) => return Err(NetMDError::UsbError(error)),
            };

            if let Some(cb) = &progress_callback {
                cb(length, done)
            }

            final_result.extend_from_slice(&res);
        }

        Ok(final_result)
    }

    pub async fn write_bulk(&mut self, data: &[u8]) -> Result<usize, NetMDError> {
        Ok(self
            .usb_interface
            .bulk_out(BULK_WRITE_ENDPOINT, data)
            .await?)
    }
}
