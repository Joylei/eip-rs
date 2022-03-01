// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use core::fmt;
use rseip_core::Error;

pub fn eip_error<U: fmt::Display, E: Error>(msg: U) -> E {
    E::custom(format_args!("Encapsulation error: {}", msg))
}

macro_rules! build_error {
    ($err_code:expr) => {
        format_args!("Encapsulation error code: {:#04x?}", $err_code)
    };
    ($err_code:expr, $detail:tt) => {
        format_args!(
            "Encapsulation error code: {:#04x?}\n\t{}",
            $err_code, $detail
        )
    };
}

#[cfg(feature = "error-explain")]
pub(crate) fn eip_error_code<E: Error>(err_code: u16) -> E {
    let msg =
        match err_code {
            0x0001 => "The sender issued an invalid or unsupported encapsulation command",
            0x0002 => "Insufficient memory resources in the receiver to handle the command",
            0x0003 => "Poorly formed or incorrect data in the data portion of the encapsulation message",
            0x0064 => "An originator used an invalid session handle when sending an encapsulation message to the target",
            0x0065 => "The target received a message of invalid length",
            0x0069 => "Unsupported encapsulation protocol revision",
            _ =>  return E::custom(build_error!(err_code)),
    };
    E::custom(build_error!(err_code, msg))
}

#[cfg(not(feature = "error-explain"))]
pub(crate) fn eip_error_code<E: Error>(err_code: u16) -> E {
    E::custom(build_error!(err_code))
}
