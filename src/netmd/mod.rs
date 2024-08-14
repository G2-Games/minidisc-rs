//! This module contains all functionality for interacting with NetMD minidisc
//! devices.

pub mod base;
pub mod commands;
pub mod encryption;
pub mod interface;
mod mappings;
mod query_utils;
mod utils;

#[doc(inline)]
pub use base::DEVICE_IDS_CROSSUSB;

#[doc(inline)]
pub use commands::NetMDContext;

#[doc(inline)]
pub use interface::NetMDInterface;

#[doc(inline)]
pub use base::NetMD;
