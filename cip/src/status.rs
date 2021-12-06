// rseip
//
// rseip - EIP&CIP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::StdResult;
use core::fmt;

/// message router reply status
#[derive(Debug, Clone, Copy)]
pub struct Status {
    pub general: u8,
    pub extended: Option<u16>,
}

impl Status {
    #[inline(always)]
    pub fn is_ok(&self) -> bool {
        self.general == 0
    }

    #[inline(always)]
    pub fn is_err(&self) -> bool {
        self.general != 0
    }

    #[inline(always)]
    pub fn is_routing_error(&self) -> bool {
        // EIP-CIP-V1-3.3  3.5.5.4
        match (self.general, self.extended) {
            (1, Some(0x0204)) => true,
            (1, Some(0x0311)) => true,
            (1, Some(0x0312)) => true,
            (1, Some(0x0315)) => true,
            (2, _) => true,
            (4, _) => true,
            _ => false,
        }
    }

    #[inline]
    pub fn into_result(self) -> StdResult<(), Status> {
        if self.general == 0 {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl fmt::Display for Status {
    #[cfg(feature = "error-explain")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.general {
            0x00 => write!(f, "Success"),
            0x01 => match self.extended {
                Some(0x0103) => write!(f, "Transport class and trigger combination not supported"),
                Some(0x0204) => write!(f, "timeout"),
                Some(0x0205) => write!(f, "Invalid SocketAddr Info item"),
                Some(0x0302) => write!(f, "Network bandwidth not available for data"),
                Some(0x0311) => write!(f, "Invalid Port ID specified in the Route_Path field"),
                Some(0x0312) => write!(f, "Invalid Node Address specified in the Route_Path field"),
                Some(0x0315) => write!(f, "Invalid segment type in the Route_Path field"),
                _ => write!(f, "Connection failure"),
            },
            0x02 => write!(f, "Resource error"), // processing unconnected send request
            0x03 => write!(f, "Bad parameter"),
            0x04 => write!(f, "Request path segment error"),
            0x05 => write!(f, "Request path destination unknown"), //Probably instance number is not present
            0x06 => write!(f, "Partial transfer"),
            0x07 => write!(f, "Connection lost"),
            0x08 => write!(f, "Service not supported"),
            0x09 => write!(f, "Invalid attribute value"),
            0x0A => write!(f, "Attribute list error"),
            0x0B => write!(f, "Already in requested mode/state"),
            0x0C => write!(f, "Object state conflict"),
            0x0D => write!(f, "Object already exists"),
            0x0E => write!(f, "Attribute not settable"),
            0x0F => write!(f, "Privilege violation"),
            0x10 => match self.extended {
                Some(0x2101) => {
                    write!(f, "Device state conflict: keyswitch position: The requestor is changing force information in HARD RUN mode")
                }
                Some(0x2802) => {
                    write!(f, "Device state conflict: Safety Status: Unable to modify Safety Memory in the current controller state")
                }
                _ => write!(f, "Device state conflict"),
            },
            0x11 => write!(f, "Reply data too large"),
            0x13 => write!(
                f,
                "Insufficient Request Data: Data too short for expected parameters"
            ),
            0x014 => write!(f, "Attribute not supported"),
            0x26 => write!(
                f,
                "The Request Path Size received was shorter or longer than expected"
            ),
            0xFF => {
                match self.extended {
                    Some(0x2104) => {
                        write!(f, "Offset is beyond end of the requested tag")
                    }
                    Some(0x2105) => {
                        write!(
                            f,
                            "Number of Elements extends beyond the end of the requested tag"
                        )
                    }
                    Some(0x2107) => {
                        write!(f, "Tag type used in request does not match the data type of the target tag")
                    }
                    _ => {
                        write!(f, "General Error: Unknown")
                    }
                }
            }
            v => write!(f, "General Error: Unknown: {:#0x}", v),
        }
    }

    #[cfg(not(feature = "error-explain"))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CIP general status: {}", self.general)?;
        if let Some(v) = self.extended {
            write!(f, ", extended status: {}", v)?;
        }
        Ok(())
    }
}

impl std::error::Error for Status {}
