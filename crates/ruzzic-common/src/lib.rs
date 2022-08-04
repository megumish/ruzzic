#![cfg_attr(not(feature = "std"), no_std)]

mod endpoint;
mod version;

pub use endpoint::EndpointType;
pub use version::QuicVersion;
#[cfg(feature = "std")]
pub use version::QuicVersions;

pub mod read_bytes_to;
pub mod var_int;
