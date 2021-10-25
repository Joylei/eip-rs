pub mod identity;
pub mod service;
pub mod socket;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revision {
    pub major: u8,
    pub minor: u8,
}
