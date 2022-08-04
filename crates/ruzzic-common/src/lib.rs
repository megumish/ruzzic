mod endpoint;
mod version;

pub use endpoint::EndpointType;
pub use version::{QuicVersion, QuicVersions};

pub mod next_bytes;
pub mod read_bytes_to;
pub mod var_int;
