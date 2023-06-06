use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::from_utf8_unchecked;

/// https://perldoc.perl.org/functions/pack
#[derive(Debug)]
pub enum PackType {
    /// A string with arbitrary binary data, will be null padded.
    StringNullPadded(Option<usize>),
    /// A text (ASCII) string, will be space padded.
    AsciiNullPadded(Option<usize>),
    /// A null-terminated (ASCIZ) string, will be null padded.
    AscizNullPadded(Option<usize>),
    // TODO: bit strings - a bit complicated
    /// A signed char (8-bit) value.
    SignedChar(Option<usize>),
    /// An unsigned char (octet) value.
    UnsignedChar(Option<usize>),
    // TODO: wchar - a bit complicated
    /// A signed short (16-bit) value.
    SignedShort(Option<usize>),
    /// An unsigned short value.
    UnsignedShort(Option<usize>),
    /// A signed long (32-bit) value.
    SignedLong(Option<usize>),
    /// An unsigned long value.
    UnsignedLong(Option<usize>),
    /// A signed quad (64-bit) value.
    SignedQuad(Option<usize>),
    /// An unsigned quad value.
    UnsignedQuad(Option<usize>),
    // TODO: integers with compile time check
    /// An unsigned short (16-bit) in "network" (big-endian) order.
    UnsignedShortBE(Option<usize>),
    /// An unsigned long (32-bit) in "network" (big-endian) order.
    UnsignedLongBE(Option<usize>),
    /// An unsigned short (16-bit) in "VAX" (little-endian) order.
    UnsignedShortLE(Option<usize>),
    /// An unsigned long (32-bit) in "VAX" (little-endian) order.
    UnsignedLongLE(Option<usize>),
    // TODO: floats are hard
    /// A null byte (a.k.a ASCII NUL, "\000", chr(0))
    NullByte(Option<usize>),
}

impl TryFrom<&str> for PackType {
    type Error = PackError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(PackError::EmptyFormatCharacter);
        }
        let size = match value.len() {
            1 => None,
            _ => {
                match value[1..].parse::<usize>() {
                    Ok(s) => Some(s),
                    Err(e) => return Err(PackError::InvalidFormatLengthArgument),
                }
            }
        };
        // https://perldoc.perl.org/functions/pack
        match value.chars().next().unwrap() { // we checked the size already
            'a' => Ok(Self::StringNullPadded(size)),
            'A' => Ok(Self::AsciiNullPadded(size)),
            'Z' => Ok(Self::AscizNullPadded(size)),
            'c' => Ok(Self::SignedChar(size)),
            'C' => Ok(Self::UnsignedChar(size)),
            's' => Ok(Self::SignedShort(size)),
            'S' => Ok(Self::UnsignedShort(size)),
            'l' => Ok(Self::SignedLong(size)),
            'L' => Ok(Self::UnsignedLong(size)),
            'q' => Ok(Self::SignedQuad(size)),
            'Q' => Ok(Self::UnsignedQuad(size)),
            'n' => Ok(Self::UnsignedShortBE(size)),
            'N' => Ok(Self::UnsignedLongBE(size)),
            'v' => Ok(Self::UnsignedShortLE(size)),
            'V' => Ok(Self::UnsignedLongLE(size)),
            'x' => Ok(Self::NullByte(size)),
            _ => Err(PackError::InvalidFormatCharacter),
        }
    }
}

impl TryFrom<String> for PackType {
    type Error = PackError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        PackType::try_from(value.as_str())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum PackError {
    LeftArgumentIsMissingForTemplate,
    RightArgumentIsMissingForTemplate,
    InvalidFormatLengthArgument,
    EmptyFormatCharacter,
    InvalidFormatCharacter,
    EmptyTemplate,
}

#[derive(Debug, Copy, Clone)]
pub enum UnpackError {}

impl Display for PackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PackError: {}", match self {
            PackError::LeftArgumentIsMissingForTemplate => "Template size is less then arguments count",
            PackError::RightArgumentIsMissingForTemplate => "Arguments count is less then template size",
            PackError::InvalidFormatLengthArgument => "Len for the argument is invalid",
            PackError::EmptyFormatCharacter => "Format character is empty",
            PackError::InvalidFormatCharacter => "Format character is not supported",
            PackError::EmptyTemplate => "Template is empty",
        })
    }
}

impl Display for UnpackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for UnpackError {}

impl Error for PackError {}

pub type Packed = Vec<u8>; // TODO: maybe some other type will fit better?

pub trait Packable {
    fn pack(self: Box<Self>, pack_type: PackType) -> Result<Packed, PackError>;
}

pub trait Unpackable {
    fn unpack(data: &[u8], pack_type: PackType) -> Result<Self, UnpackError> where Self: Sized;
}

pub struct PackableArg {
    inner: Box<dyn Packable>,
}

pub fn pack<T>(template: &str, args: T) -> Result<Packed, PackError> where
    T: Iterator<Item=PackableArg> {
    // very stupid version
    // one day I will write something better
    if template.is_empty() {
        return Err(PackError::EmptyTemplate);
    }
    let mut packed_template: Vec<PackType> = Vec::with_capacity(template.len()); // predict
    let binding = template.chars().filter(|f| f.is_ascii_alphanumeric()).collect::<String>();
    let t = binding.as_bytes();
    let mut end = t.len();
    let mut start = t.len() - 1;
    loop {
        if t[start].is_ascii_alphabetic() {
            let f = &t[start..end];
            packed_template.push(PackType::try_from(unsafe { from_utf8_unchecked(f) })?); // it's safe as we just converted it from valid utf8
            end = start;
        }
        if start == 0 {
            break;
        }
        start -= 1;
    }
    pack_private(packed_template.into_iter().rev(), args)
}

fn pack_private<X, T>(mut template: X, mut args: T) -> Result<Packed, PackError> where
    X: Iterator<Item=PackType>,
    T: Iterator<Item=PackableArg> {
    let mut result = Packed::with_capacity(4096); // TODO: 4k slab is okay or not?
    loop {
        let packaging = template.next();
        let argument = args.next();
        match (packaging, argument) {
            (Some(p), Some(a)) => {
                match a.inner.pack(p) {
                    Ok(mut data) => {
                        result.append(&mut data);
                    }
                    Err(e) => return Err(e),
                }
            }
            (None, Some(_)) => {
                return Err(PackError::LeftArgumentIsMissingForTemplate);
            }
            (Some(_), None) => {
                return Err(PackError::RightArgumentIsMissingForTemplate);
            }
            (None, None) => {
                return Ok(result);
            }
        }
    }
}

pub fn unpack<T>(template: &str, packed: Packed) -> Result<T, UnpackError>
    where T: Iterator<Item=dyn Unpackable> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack() {
        impl Packable for u16 {
            fn pack(self: Box<Self>, pack_type: PackType) -> Result<Packed, PackError> {
                match pack_type {
                    PackType::StringNullPadded(Some(10)) => Ok(vec![0, 10]),
                    PackType::UnsignedShort(Some(3)) => Ok(vec![33, 3]),
                    PackType::SignedShort(None) => Ok(vec![44, 44]),
                    _ => Err(PackError::InvalidFormatCharacter)
                }
            }
        }
        let pack = pack("a[10]S3s", [10u16, 11u16, 12u16].map(|f| PackableArg { inner: Box::new(f) }).into_iter());
        assert!(pack.is_ok());
        assert!(pack.unwrap().eq(&[0, 10, 33, 3, 44, 44u8]));
    }
}

