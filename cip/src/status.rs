// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use core::fmt;

/// message reply status
#[derive(Debug, Clone, Copy)]
pub struct Status {
    pub general: u8,
    pub extended: Option<u16>,
}

impl Status {
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.general == 0
    }

    #[inline]
    pub fn is_err(&self) -> bool {
        self.general != 0
    }

    #[inline]
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
    pub fn into_result(self) -> Result<(), Status> {
        if self.general == 0 {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CIP general status: {}", self.general)?;
        if let Some(v) = self.extended {
            write!(f, ", extended status: {}", v)?;
        }
        #[cfg(feature = "error-explain")]
        {
            let msg = match self.general {
                0x00 => return Ok(()),
                0x01 => match self.extended {
                    Some(0x0103) => "Transport class and trigger combination not supported",
                    Some(0x0204) => "timeout",
                    Some(0x0205) => "Invalid SocketAddr Info item",
                    Some(0x0302) => "Network bandwidth not available for data",
                    Some(0x0311) => "Invalid Port ID specified in the Route_Path field",
                    Some(0x0312) => "Invalid Node Address specified in the Route_Path field",
                    Some(0x0315) =>"Invalid segment type in the Route_Path field",
                    _ => "Connection failure",
                },
                0x02 => "Resource error", // processing unconnected send request
                0x03 => "Bad parameter",
                0x04 => "Request path segment error",
                0x05 => "Request path destination unknown", //Probably instance number is not present
                0x06 => "Partial transfer",
                0x07 => "Connection lost",
                0x08 => "Service not supported",
                0x09 => "Invalid attribute value",
                0x0A => "Attribute list error",
                0x0B => "Already in requested mode/state",
                0x0C => "Object state conflict",
                0x0D => "Object already exists",
                0x0E => "Attribute not settable",
                0x0F => "Privilege violation",
                0x10 => match self.extended {
                    Some(0x2101) => "Device state conflict: keyswitch position: The requestor is changing force information in HARD RUN mode",
                    Some(0x2802) => "Device state conflict: Safety Status: Unable to modify Safety Memory in the current controller state",
                    _ => "Device state conflict",
                },
                0x11 => "Reply data too large",
                0x13 => "Insufficient Request Data: Data too short for expected parameters",
                0x014 => "Attribute not supported",
                0x26 => "The Request Path Size received was shorter or longer than expected",
                0xFF => match self.extended {
                        Some(0x2104) =>  "Offset is beyond end of the requested tag",
                        Some(0x2105) =>  "Number of Elements extends beyond the end of the requested tag",
                        Some(0x2107) =>  "Tag type used in request does not match the data type of the target tag",
                        _ =>  "General Error: Unknown",
                },
                _ => "General Error: Unknown",
            };
            write!(f, "\n\t{}", msg)?;
        }
        Ok(())
    }
}

impl std::error::Error for Status {}
