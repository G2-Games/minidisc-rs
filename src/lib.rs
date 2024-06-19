//! A crate for controlling NetMD and Hi-MD devices.
//!
//! This crate is entirely `async` (a necessity because of USB in WASM), but
//! it can be used in programs which are not async by using a crate like
//! [futures_lite](https://docs.rs/futures-lite/) with the `block_on` function.
//!
//! To use this library, first you need to get a device from [`cross_usb`] and
//! then open a [`NetMDContext`].
//!
//! ```rust
//! use cross_usb::prelude::get_device;
//! use minidisc::netmd::base::DEVICE_IDS_CROSSUSB;
//! use minidisc::NetMDContext;
//!
//! // Get a device using the built-in list of descriptors for minidisc devices
//! let dev_descriptor = cross_usb::get_device(DEVICE_IDS_CROSSUSB).await
//!     .expect("Failed to find device");
//!
//! // Open a NetMD Context with the device
//! let context = NetMDContext::new(dev_descriptor).await
//!     .expect("Could not create context");
//!
//! // Perform operations on it ...
//! context.list_content().await
//!     .expect("Could not list disc contents");
//! ```

pub mod netmd;

#[doc(inline)]
pub use netmd::commands::NetMDContext;
