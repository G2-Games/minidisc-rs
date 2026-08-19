//! This module contains all functionality for interacting with NetMD minidisc
//! devices.

pub mod base;
pub mod commands;
pub mod encryption;
pub mod interface;
mod mappings;
mod query_utils;
pub mod utils;

#[doc(inline)]
pub use interface::NetMD;

#[doc(inline)]
pub use base::NetMDBase;
