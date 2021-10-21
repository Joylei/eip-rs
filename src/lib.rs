pub mod client;
mod codec;
pub mod consts;
pub mod error;
pub mod frame;
pub mod objects;

pub type Result<T> = std::result::Result<T, error::Error>;
