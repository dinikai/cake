pub mod auth;
pub mod checksum;
pub mod cmd;
pub mod config;
pub mod errors;
pub mod proto;
pub mod token_pool;

// There is a separate protocol version type for the future extensibility.
// For example, this type could later mutate into a struct that would
// carry not just a single number, but the major and minor versions.
pub type ProtocolVer = u32;

pub const PROTOCOL_VER: u32 = 1;
