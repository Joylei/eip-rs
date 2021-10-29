// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2020-2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::codec::Encodable;

/// EIP Command
pub trait Command: Encodable {
    fn command_code() -> u16;
}

#[derive(Debug, Default)]
pub struct Nop<D> {
    pub data: D,
}

impl<D: Encodable> Command for Nop<D> {
    #[inline(always)]
    fn command_code() -> u16 {
        0x0000
    }
}

#[derive(Debug)]
pub struct ListIdentity;

impl Command for ListIdentity {
    #[inline(always)]
    fn command_code() -> u16 {
        0x0063
    }
}

#[derive(Debug)]
pub struct ListInterfaces;

impl Command for ListInterfaces {
    #[inline(always)]
    fn command_code() -> u16 {
        0x0064
    }
}

#[derive(Debug)]
pub struct ListServices;

impl Command for ListServices {
    #[inline(always)]
    fn command_code() -> u16 {
        0x0004
    }
}

#[derive(Debug)]
pub struct RegisterSession;

impl Command for RegisterSession {
    #[inline(always)]
    fn command_code() -> u16 {
        0x0065
    }
}

#[derive(Debug)]
pub struct UnRegisterSession {
    pub session_handle: u32,
}

impl Command for UnRegisterSession {
    #[inline(always)]
    fn command_code() -> u16 {
        0x0066
    }
}

/// for UCMM (unconnected message), sent by originator
#[derive(Debug)]
pub struct SendRRData<D> {
    pub session_handle: u32,
    /// operation timeout, in seconds;
    /// - set to 0, rely on the timeout mechanism of the encapsulated protocol
    /// - usually set to 0 for CIP
    pub timeout: u16,
    /// Data to be Sent via Unconnected Message
    pub data: D,
}

impl<D: Encodable> Command for SendRRData<D> {
    #[inline(always)]
    fn command_code() -> u16 {
        0x006F
    }
}

/// for connected message, sent by either end, no reply
#[derive(Debug)]
pub struct SendUnitData<D> {
    pub session_handle: u32,
    pub connection_id: u32,
    pub sequence_number: u16,
    /// Data to be Sent via Connected Message
    pub data: D,
}

impl<D: Encodable> Command for SendUnitData<D> {
    #[inline(always)]
    fn command_code() -> u16 {
        0x0070
    }
}
