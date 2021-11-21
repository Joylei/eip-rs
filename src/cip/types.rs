// rseip
//
// rseip (eip-rs) - EtherNet/IP in pure Rust.
// Copyright: 2021, Joylei <leingliu@gmail.com>
// License: MIT

/// type code =  0xC1
pub type BOOL = bool;

/// type code = 0xC2
pub type SINT = i8;

/// type code = 0xC3
pub type INT = i16;

/// type code = 0xC4
pub type DINT = i32;

/// type code = 0xC5
pub type LINT = i64;

/// type code = 0xC6
pub type USINT = u8;

/// type code = 0xC7
pub type UINT = u16;

/// type code = 0xC8
pub type UDINT = u32;

/// type code =0xC9
pub type ULINT = u64;

/// type code = 0xCA
pub type REAL = f32;

/// type code = 0xCB
pub type LREAL = f64;

// STIME = 0xCC

// SDATE = 0xCD

// TIME_OF_DAY = 0xCE

// STRING = 0xD0

/// type code = 0xD1
pub type BYTE = u8;

/// type code = 0xD2
pub type WORD = u16;

/// type code = 0xD3
pub type DWORD = u32;

/// type code = 0xD4
pub type LWORD = u64;

// STRING2 = 0xD5, wide string

// FTIME = 0xD6, duration (high resolution)

// LTIME= 0xD7, duration (long)

// ITIME = 0xD8, duration (short)

// STRINGN = 0xD9, N bytes per character

// SHORT_STRING = 0xDA     len | chars

// TIME = 0xDB, duration (milliseconds)

// EPATH = 0xDC

// ENGUNIT = 0xDD, engineering units
