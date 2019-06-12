//! ILP address types.
//!
//! Reference: [ILP Addresses - v2.0.0](https://github.com/interledger/rfcs/blob/master/0015-ilp-addresses/0015-ilp-addresses.md).

// Addresses are never empty.
#![allow(clippy::len_without_is_empty)]

use regex::Regex;
use std::error;
use std::fmt;
use std::str;

use crate::errors::ParseError;
use bytes::{BufMut, Bytes, BytesMut};
use std::convert::TryFrom;
use std::str::FromStr;

use lazy_static::lazy_static;

const MAX_ADDRESS_LENGTH: usize = 1023;

#[derive(Debug)]
pub enum AddressError {
    InvalidLength(usize),
    InvalidFormat,
}

lazy_static! {
    static ref ADDRESS_PATTERN: Regex =
        Regex::new(r"^(g|private|example|peer|self|test[1-3]?|local)([.][a-zA-Z0-9_~-]+)+$")
            .unwrap();
}


use std::error::Error;
impl Error for AddressError {
    fn description(&self) -> &str {
        match *self {
            AddressError::InvalidLength(length) => "invalid address length",
            AddressError::InvalidFormat => "invalid address format",
        }
    }
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

/// An ILP address backed by `Bytes`.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Address<'a>(&'a Bytes);

impl<'a> FromStr for Address<'a> {
    type Err = ParseError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Address::try_from(&Bytes::from(src))
    }
}

impl<'a> TryFrom<&'a Bytes> for Address<'a> {
    type Error = ParseError;

    fn try_from(bytes: &'a Bytes) -> Result<Address<'a>, Self::Error> {
        // https://interledger.org/rfcs/0015-ilp-addresses/#address-requirements
        if bytes.len() > MAX_ADDRESS_LENGTH {
            return Err(ParseError::InvalidAddress(AddressError::InvalidLength(
                bytes.len(),
            )));
        }

        if ADDRESS_PATTERN.is_match(str::from_utf8(&bytes)?) {
            Ok(Address(bytes))
        } else {
            Err(ParseError::InvalidAddress(AddressError::InvalidFormat))
        }
    }
}

// impl<'a> TryFrom<Bytes> for Address<'a> {
//     type Error = ParseError;
// 
//     fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
//         // https://interledger.org/rfcs/0015-ilp-addresses/#address-requirements
//         if bytes.len() > MAX_ADDRESS_LENGTH {
//             return Err(ParseError::InvalidAddress(AddressError::InvalidLength(
//                 bytes.len(),
//             )));
//         }
// 
//         if ADDRESS_PATTERN.is_match(str::from_utf8(&bytes)?) {
//             Ok(Address(&bytes))
//         } else {
//             Err(ParseError::InvalidAddress(AddressError::InvalidFormat))
//         }
//     }
// }

impl<'a> TryFrom<&[u8]> for Address<'a> {
    type Error = ParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from(&Bytes::from(bytes))
    }
}

impl <'a> std::ops::Deref for Address<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.0.as_ref()) }
    }
}

impl <'a> AsRef<[u8]> for Address<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl <'a> AsRef<Bytes> for Address<'a> {
    #[inline]
    fn as_ref(&self) -> &Bytes {
        &self.0
    }
}

impl <'a> fmt::Debug for Address<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_tuple("Address")
            .field(&self.to_string())
            .finish()
    }
}

impl <'a> fmt::Display for Address<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self)
    }
}

impl<'a> Address<'a> {
    /// Returns the length of the ILP Address.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the `Bytes` conversion of the ILP Address
    pub fn to_bytes(&self) -> Bytes {
        self.0.clone()
    }

    /// Creates an ILP address without validating the bytes.
    ///
    /// # Safety
    ///
    /// The given bytes must be a valid ILP address.
    #[inline]
    pub unsafe fn new_unchecked(bytes: &'a Bytes) -> Self {
        debug_assert!(Address::try_from(bytes.as_ref()).is_ok());
        Address(&bytes)
    }

    /// ```text
    /// scheme = "g" / "private" / "example" / "peer" / "self" /
    ///          "test" / "test1" / "test2" / "test3" / "local"
    /// ```
    #[inline]
    pub fn scheme(&self) -> &str {
        self.segments().next().unwrap()
    }

    /// Returns an iterator over all the segments of the ILP Address
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        unsafe {
            self.0
                .split(|&b| b == b'.')
                .map(|s| str::from_utf8_unchecked(&s))
        }
    }

    /// Returns the local part (right-most '.' separated segment) of the ILP Address.
    pub fn local(&self) -> &[u8] {
        self.0.rsplit(|&byte| byte == b'.').next().unwrap()
    }

    /// Suffixes the ILP Address with the provided suffix. Includes a '.' separator
    pub fn with_suffix(&self, suffix: &[u8]) -> Result<Address, ParseError> {
        let new_address_len = self.len() + 1 + suffix.len();
        let mut new_address = BytesMut::with_capacity(new_address_len);

        new_address.put_slice(self.0.as_ref());
        new_address.put(b'.');
        new_address.put_slice(suffix);

        Address::try_from(&new_address.freeze())
    }
}

impl<'a> PartialEq<[u8]> for Address<'a> {
    fn eq(&self, other: &[u8]) -> bool {
        self.0 == other
    }
}

#[cfg(any(feature = "serde", test))]
impl<'de> serde::Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = <&str>::deserialize(deserializer)?;
        Address::from_str(string).map_err(serde::de::Error::custom)
    }
}

#[cfg(any(feature = "serde", test))]
impl serde::Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&*self)
    }
}

#[cfg(test)]
mod test_address {
    use serde::ser::{Serialize, SerializeStruct, Serializer};
    use serde_test::{
        assert_de_tokens, assert_de_tokens_error, assert_ser_tokens, assert_ser_tokens_error, Token,
    };

    use super::*;

    static VALID_ADDRESSES: &'static [&'static [u8]] = &[
        b"test.alice.XYZ.1234.-_~",
        b"g.us-fed.ach.0.acmebank.swx0a0.acmecorp.sales.199.~ipr.cdfa5e16-e759-4ba3-88f6-8b9dc83c1868.2",

        b"g.A", b"private.A", b"example.A", b"peer.A", b"self.A",
        b"test.A", b"test1.A", b"test2.A", b"test3.A", b"local.A",
    ];

    static INVALID_ADDRESSES: &'static [&'static [u8]] = &[
        b"", // empty
        // Invalid characters.
        b"test.alice 123",
        b"test.alice!123",
        b"test.alice/123",
        // Bad schemes.
        b"test",        // only a scheme
        b"what.alice",  // invalid scheme
        b"test4.alice", // invalid scheme
        // Invalid separators.
        b"test.",       // only a prefix
        b"test.alice.", // ends in a separator
        b".test.alice", // begins with a separator
        b"test..alice", // double separator
    ];

    #[test]
    fn test_try_from() {
        for address in VALID_ADDRESSES {
            assert_eq!(
                Address::try_from(*address).unwrap(),
                Address(Bytes::from(*address)),
                "address: {:?}",
                String::from_utf8_lossy(address),
            );
        }

        let longest_address = &make_address(1023)[..];
        assert_eq!(
            Address::try_from(longest_address).unwrap(),
            Address(Bytes::from(longest_address)),
        );

        for address in INVALID_ADDRESSES {
            assert!(
                Address::try_from(*address).is_err(),
                "address: {:?}",
                String::from_utf8_lossy(address),
            );
        }

        let too_long_address = &make_address(1024)[..];
        assert!(Address::try_from(too_long_address).is_err());
    }

    #[test]
    fn test_deserialize() {
        assert_de_tokens(
            &Address::try_from(Bytes::from("test.alice")).unwrap(),
            &[Token::BorrowedStr("test.alice")],
        );
        assert_de_tokens_error::<Address>(
            &[Token::BorrowedStr("test.alice ")],
            "invalid address format",
        );
    }

    #[test]
    fn test_serialize() {
        let addr = Address::try_from(Bytes::from("test.alice")).unwrap();
        assert_ser_tokens(&addr, &[Token::Str("test.alice")]);
    }

    #[test]
    fn test_len() {
        assert_eq!(
            Address::from_str("test.alice").unwrap().len(),
            "test.alice".len(),
        );
    }

    #[test]
    fn test_scheme() {
        assert_eq!(Address::from_str("test.alice").unwrap().scheme(), "test",);
        assert_eq!(
            Address::from_str("test.alice.1234").unwrap().scheme(),
            "test",
        );
    }

    #[test]
    fn test_with_suffix() {
        assert_eq!(
            Address::from_str("test.alice")
                .unwrap()
                .with_suffix(b"1234")
                .unwrap(),
            Address::from_str("test.alice.1234").unwrap(),
        );
        assert!({
            Address::from_str("test.alice")
                .unwrap()
                .with_suffix(b"12 34")
                .is_err()
        });
    }

    #[test]
    fn test_debug() {
        assert_eq!(
            format!("{:?}", Address::from_str("test.alice").unwrap()),
            "Address(\"test.alice\")",
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(
            format!("{}", Address::from_str("test.alice").unwrap()),
            "test.alice",
        );
    }

    fn make_address(length: usize) -> Vec<u8> {
        let mut addr = b"test.".to_vec();
        addr.resize(length, b'_');
        addr
    }
}
