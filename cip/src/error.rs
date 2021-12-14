// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::*;
use core::fmt;
use rseip_core::Error;

pub fn cip_error<U: fmt::Display, E: Error>(msg: U) -> E {
    E::custom(format_args!("cip error: {}", msg))
}

pub fn cip_error_status<E: Error>(status: Status) -> E {
    E::custom(format_args!("cip error: message reply status {}", status))
}

pub fn cip_error_reply<E: Error>(reply_service: u8, expected_service: u8) -> E {
    E::custom(format_args!(
        "cip error: unexpected message reply service {}, expect {}",
        reply_service, expected_service
    ))
}
