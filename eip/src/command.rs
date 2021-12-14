// rseip
//
// rseip - Ethernet/IP (CIP) in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

use crate::consts::*;
use rseip_core::codec::Encode;

/// encapsulation command
pub trait Command: Encode {
    fn command_code() -> u16;
}

/// NOP command
#[derive(Debug, Default)]
pub struct Nop<D> {
    pub data: D,
}

impl<D: Encode> Command for Nop<D> {
    #[inline(always)]
    fn command_code() -> u16 {
        EIP_COMMAND_NOP
    }
}

/// List_Identity command
#[derive(Debug)]
pub struct ListIdentity;

impl Command for ListIdentity {
    #[inline(always)]
    fn command_code() -> u16 {
        EIP_COMMAND_LIST_IDENTITY
    }
}

/// ListInterface command
#[derive(Debug)]
pub struct ListInterfaces;

impl Command for ListInterfaces {
    #[inline(always)]
    fn command_code() -> u16 {
        EIP_COMMAND_LIST_INTERFACES
    }
}

/// ListService command
#[derive(Debug)]
pub struct ListServices;

impl Command for ListServices {
    #[inline(always)]
    fn command_code() -> u16 {
        EIP_COMMAND_LIST_SERVICE
    }
}

/// RegisterSession command
#[derive(Debug)]
pub struct RegisterSession;

impl Command for RegisterSession {
    #[inline(always)]
    fn command_code() -> u16 {
        EIP_COMMAND_REGISTER_SESSION
    }
}

/// UnRegisterSession command
#[derive(Debug)]
pub struct UnRegisterSession {
    pub session_handle: u32,
}

impl Command for UnRegisterSession {
    #[inline(always)]
    fn command_code() -> u16 {
        EIP_COMMAND_UNREGISTER_SESSION
    }
}

/// SendRRData command, for UCMM (unconnected message), sent by originator
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

impl<D: Encode> Command for SendRRData<D> {
    #[inline(always)]
    fn command_code() -> u16 {
        EIP_COMMAND_SEND_RRDATA
    }
}

/// SendUnitData command, for connected message, sent by either end, no reply
#[derive(Debug)]
pub struct SendUnitData<D> {
    pub session_handle: u32,
    pub connection_id: u32,
    pub sequence_number: u16,
    /// Data to be Sent via Connected Message
    pub data: D,
}

impl<D: Encode> Command for SendUnitData<D> {
    #[inline(always)]
    fn command_code() -> u16 {
        EIP_COMMAND_SEND_UNIT_DATA
    }
}
