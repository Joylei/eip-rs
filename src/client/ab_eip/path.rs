use crate::cip::epath::{EPath, Segment};
use core::fmt;
use core::str;

pub enum PathError {
    Empty,
    UnexpectedByte(u8),
    SyntaxError,
    NumberParseError,
    NameTooLong,
    NameParseError,
    Eof,
}

impl fmt::Debug for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "input buffer is empty"),
            Self::UnexpectedByte(c) => write!(f, "syntax error - unexpected byte: {:#02x?}", c),
            Self::SyntaxError => write!(f, "syntax error"),
            Self::NumberParseError => write!(f, "syntax error - parse number failure"),
            Self::NameTooLong => write!(f, "syntax error - name too long"),
            Self::NameParseError => write!(f, "syntax error - parse name failure"),
            Self::Eof => write!(f, "syntax error - unexpected end of input buffer"),
        }
    }
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl std::error::Error for PathError {}

/// AB tag path parser
pub trait PathParser: Sized {
    /// parse tag path
    fn parse_tag(path: impl AsRef<[u8]>) -> Result<Self, PathError>;
}

impl PathParser for EPath {
    /// parse tag path
    ///
    /// tag path examples
    /// - `struct_a`
    /// - `struct_a.1`
    /// - `profile[0,1,257]`
    /// - `a.b.c`
    /// - `struct_a[1].a.b[1]`
    ///  - `Program:MainProgram.test`
    ///
    #[inline]
    fn parse_tag(path: impl AsRef<[u8]>) -> Result<Self, PathError> {
        let mut buf = path.as_ref();
        if buf.is_empty() {
            return Err(PathError::Empty);
        }
        let mut res = EPath::default();
        parse_symbol_and_optional_numbers(&mut buf, &mut res, true)?;
        while let Some(c) = buf.first() {
            match c {
                &b'.' => {
                    buf = &buf[1..];
                    // 2 cases
                    match buf.first() {
                        Some(c) if is_digit(*c) => {
                            // case 1: bool member
                            let num = parse_number(&mut buf)?;
                            // no remaining
                            if !buf.is_empty() {
                                return Err(PathError::SyntaxError);
                            }
                            // 0-7 or 0-31 if DWORD
                            if num >= 32 {
                                return Err(PathError::SyntaxError);
                            }
                            res.push(Segment::Element(num));
                        }
                        Some(_) => {
                            // case 2
                            parse_symbol_and_optional_numbers(&mut buf, &mut res, false)?;
                        }
                        None => return Err(PathError::Eof),
                    }
                }
                c => return Err(PathError::UnexpectedByte(*c)),
            }
        }
        Ok(res)
    }
}

#[inline]
fn parse_symbol_and_optional_numbers(
    buf: &mut &[u8],
    res: &mut EPath,
    allow_colon: bool,
) -> Result<(), PathError> {
    let sym = parse_symbol(buf, allow_colon)?;
    res.push(Segment::Symbol(sym.into()));
    if let Some(&b'[') = buf.first() {
        *buf = &buf[1..];
        parse_numbers(buf, res)?;
    }
    Ok(())
}

#[inline]
fn parse_numbers(buf: &mut &[u8], res: &mut EPath) -> Result<(), PathError> {
    let mut count = 0;
    loop {
        let idx = parse_number(buf)?;
        res.push(Segment::Element(idx));
        count += 1;
        match buf.first() {
            Some(b',') => {
                *buf = &buf[1..];
            }
            Some(b']') => {
                *buf = &buf[1..];
                break;
            }
            Some(c) => {
                return Err(PathError::UnexpectedByte(*c));
            }
            _ => return Err(PathError::Eof),
        }
    }

    if count > 0 {
        Ok(())
    } else {
        Err(PathError::SyntaxError)
    }
}

#[inline]
fn parse_number(buf: &mut &[u8]) -> Result<u32, PathError> {
    const MAX_LEN: usize = 10; // u32::MAX = 4294967295
    check_eof(buf)?;
    let digits_buf = take_one_plus(buf, is_digit)
        .and_then(|v| if v.len() > MAX_LEN { None } else { Some(v) })
        .ok_or(PathError::NumberParseError)?;

    // safety: all digits
    let text = unsafe { str::from_utf8_unchecked(digits_buf) };
    let num = text.parse().map_err(|_| PathError::NumberParseError)?;
    Ok(num)
}

#[inline]
fn parse_symbol<'a>(buf: &'a mut &[u8], allow_colon: bool) -> Result<&'a str, PathError> {
    const MAX_LEN: usize = 40; // see 1756-pm020_-en-p.pdf  page 12

    //check first byte
    buf.first().map_or_else(
        || Err(PathError::Eof),
        |c| {
            if *c == b'_' || is_alphabet(*c) {
                Ok(())
            } else {
                Err(PathError::NameParseError)
            }
        },
    )?;

    let name_buf = if allow_colon && has_program(buf) {
        let temp = &buf[..];
        *buf = &buf[8..];
        take_one_plus(buf, is_valid_char).map(|v| &temp[..8 + v.len()])
    } else {
        take_one_plus(buf, is_valid_char)
    };
    let name_buf = name_buf.map_or_else(
        || Err(PathError::NameParseError),
        |v| {
            if v.len() > MAX_LEN {
                Err(PathError::NameTooLong)
            } else {
                Ok(v)
            }
        },
    )?;

    // safety: all ASCII
    let name = unsafe { str::from_utf8_unchecked(name_buf) };
    Ok(name)
}

// === chars ====

/// check program prefix ignore case
#[inline]
fn has_program(buf: &[u8]) -> bool {
    if buf.len() >= 8 {
        let temp = unsafe { str::from_utf8_unchecked(&buf[..8]) };
        temp.eq_ignore_ascii_case("program:")
    } else {
        false
    }
}

#[inline]
fn take_one_plus<'a>(buf: &mut &'a [u8], f: impl FnMut(u8) -> bool) -> Option<&'a [u8]> {
    if let Some((first, rest)) = split(buf, f) {
        *buf = rest;
        Some(first)
    } else {
        None
    }
}

/// split buf if `pred` matches more than one bytes
#[inline]
fn split(buf: &[u8], mut pred: impl FnMut(u8) -> bool) -> Option<(&[u8], &[u8])> {
    if !buf.is_empty() {
        let mut i = 0;
        for c in buf.iter() {
            if !pred(*c) {
                break;
            }
            i += 1;
        }
        if i > 0 {
            return Some(buf.split_at(i));
        }
    }
    None
}
#[inline]
const fn check_eof(buf: &[u8]) -> Result<(), PathError> {
    if buf.is_empty() {
        Err(PathError::Eof)
    } else {
        Ok(())
    }
}

#[inline]
const fn is_valid_char(c: u8) -> bool {
    c == b'_' || is_digit(c) || is_alphabet(c)
}

#[inline]
const fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

#[inline]
const fn is_alphabet(c: u8) -> bool {
    if c >= b'a' && c <= b'z' {
        true
    } else {
        c >= b'A' && c <= b'Z'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tag_paths() {
        let path = EPath::parse_tag("struct_a").unwrap();
        assert_eq!(path, EPath::from_symbol("struct_a"));

        let path = EPath::parse_tag("_under").unwrap();
        assert_eq!(path, EPath::from_symbol("_under"));

        let path = EPath::parse_tag("struct_a.1").unwrap();
        assert_eq!(path, EPath::from_symbol("struct_a").with_element(1));

        let path = EPath::parse_tag("profile[0,1,257]").unwrap();
        assert_eq!(
            path,
            EPath::from_symbol("profile")
                .with_element(0)
                .with_element(1)
                .with_element(257)
        );

        let path = EPath::parse_tag("a.b.c").unwrap();
        assert_eq!(
            path,
            EPath::from_symbol("a").with_symbol("b").with_symbol("c")
        );

        let path = EPath::parse_tag("ProGram:MainProgram.test").unwrap();
        assert_eq!(
            path,
            EPath::from_symbol("ProGram:MainProgram").with_symbol("test")
        );

        let path = EPath::parse_tag("struct_a[1]._abc.efg[2,3]").unwrap();
        assert_eq!(
            path,
            EPath::from_symbol("struct_a")
                .with_element(1)
                .with_symbol("_abc")
                .with_symbol("efg")
                .with_element(2)
                .with_element(3)
        );
    }

    #[test]
    fn test_invalid_tag_paths() {
        let paths = [
            "",
            ".",
            "[",
            "124",
            "123534546456565756",
            "_abc-",
            ".1234",
            "[12345]",
            "abc[1,]",
            "abc[1,3,]",
            "abc[1,3",
            "abc[1,3,",
            "my.heart:on",
        ];

        for item in paths {
            let res = EPath::parse_tag(item);
            assert!(res.is_err());
        }
    }
}
