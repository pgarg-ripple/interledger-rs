use std::error;
use std::fmt;

use chrono;
use std::io;
use std::str;
use std::string;


#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Utf8Error(str::Utf8Error),
    FromUtf8Error(string::FromUtf8Error),
    Chrono(chrono::ParseError),
    InvalidPacket(String),
    #[allow(dead_code)]
    UnexpectedPacket(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => fmt::Display::fmt(e, f),
            Error::Utf8Error(ref e) => fmt::Display::fmt(e, f),
            Error::FromUtf8Error(ref e) => fmt::Display::fmt(e, f),
            Error::Chrono(ref e) => fmt::Display::fmt(e, f),
            Error::InvalidPacket(ref msg) => write!(f, "invalid packet: {}", msg),
            Error::UnexpectedPacket(ref msg) => write!(f, "unexpected packet: {}", msg),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::Utf8Error(ref e) => e.description(),
            Error::FromUtf8Error(ref e) => e.description(),
            Error::Chrono(ref e) => e.description(),
            Error::InvalidPacket(..) => "structurally invalid packet",
            Error::UnexpectedPacket(..) => "unexpected packet",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref e) => Some(e),
            Error::Utf8Error(ref e) => Some(e),
            Error::FromUtf8Error(ref e) => Some(e),
            Error::Chrono(ref e) => Some(e),
            Error::InvalidPacket(..) | Error::UnexpectedPacket(..) => None,
        }
    }
}

#[doc(hidden)]
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

#[doc(hidden)]
impl From<str::Utf8Error> for Error {
    fn from(e: str::Utf8Error) -> Error {
        Error::Utf8Error(e)
    }
}

#[doc(hidden)]
impl From<string::FromUtf8Error> for Error {
    fn from(e: string::FromUtf8Error) -> Error {
        Error::FromUtf8Error(e)
    }
}

#[doc(hidden)]
impl From<chrono::ParseError> for Error {
    fn from(e: chrono::ParseError) -> Error {
        Error::Chrono(e)
    }
}