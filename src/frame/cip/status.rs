// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use std::fmt;

/// message router reply status
#[derive(Debug, Clone)]
pub struct Status {
    pub general: u8,
    pub extended: u16,
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
            (1, 0x0204) => true,
            (1, 0x0311) => true,
            (1, 0x0312) => true,
            (1, 0x0315) => true,
            (2, _) => true,
            (4, _) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.general {
                0x00 => write!(f, "Success"),
                0x01 => {
                    match self.extended {
                        0x0103 => write!(f, "Transport class and trigger combination not supported"),
                        0x0204 => write!(f, "timeout"),
                        0x0205 => write!(f, "Invalid SocketAddr Info item"),
                        0x0302 => write!(f, "Network bandwidth not available for data"),
                        0x0311 =>write!(f, "Invalid Port ID specified in the Route_Path field"),
                        0x0312 => write!(f, "Invalid Node Address specified in the Route_Path field"),
                        0x0315 => write!(f, "Invalid segment type in the Route_Path field"),
                        _ =>write!(f, "General Error: Unknown")
                    }
                }
                0x02 => write!(f, "Resource error"), // processing unconnected send request
                0x03 => write!(f, "Bad parameter, size > 12 or size greater than size of element"),
                0x04 => write!(f, "A syntax error was detected decoding the Request Path"),
                0x05 => write!(f, "Request Path destination unknown: Probably instance number is not present"),
                0x06 => write!(f, "Insufficient Packet Space: Not enough room in the response buffer for all the data"),
                0x10 => {
                    match self.extended {
                        0x2101 => {
                            write!(f, "Device state conflict: keyswitch position: The requestor is changing force information in HARD RUN mode")
                        },
                        0x2802 => {
                            write!(f, "Device state conflict: Safety Status: Unable to modify Safety Memory in the current controller state")
                        }
                        _=>write!(f, "General Error: Unknown")
                    }
                }
                0x13 => write!(f, "Insufficient Request Data: Data too short for expected parameters"),
                0x26 => write!(f, "The Request Path Size received was shorter or longer than expected"),
                0xFF => {
                    match self.extended {
                        0x2104 => {
                            write!(f, "General Error: Offset is beyond end of the requested tag")
                        }
                        0x2105 => {
                            write!(f, "General Error: Number of Elements extends beyond the end of the requested tag")
                        },
                        0x2107 => {
                            write!(f, "General Error: Tag type used in request does not match the data type of the target tag")
                        },
                        _ => {
                            write!(f, "General Error: Unknown")
                        }
                    }
                },
                _ => write!(f, "General Error: Unknown")
            }
    }
}
