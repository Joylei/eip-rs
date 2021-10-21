mod codec;
pub mod consts;
pub mod error;
mod frame;
mod objects;

pub type Result<T> = std::result::Result<T, error::Error>;
