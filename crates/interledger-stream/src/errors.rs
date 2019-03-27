use std::error;
use std::fmt;

use std::io;
use std::str;
use std::string;
use interledger_packet;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Utf8Error(str::Utf8Error),
    FromUtf8Error(string::FromUtf8Error),
    InvalidPacket(String),
    UnexpectedPacket(String),
    IlpError(interledger_packet::Error),
    ConnectionError(String),
    PollError(String),
    SendMoneyError(String),

}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => fmt::Display::fmt(e, f),
            Error::Utf8Error(ref e) => fmt::Display::fmt(e, f),
            Error::FromUtf8Error(ref e) => fmt::Display::fmt(e, f),
            Error::InvalidPacket(ref msg) => write!(f, "invalid packet: {}", msg),
            Error::UnexpectedPacket(ref msg) => write!(f, "unexpected packet: {}", msg),
            Error::IlpError(ref e) => fmt::Display::fmt(e, f),
            Error::ConnectionError(ref msg) => write!(f, "connection error: {}", msg),
            Error::PollError(ref msg) => write!(f, "poll error: {}", msg),
            Error::SendMoneyError(ref msg) => write!(f, "error sending money: {}", msg),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::Utf8Error(ref e) => e.description(),
            Error::FromUtf8Error(ref e) => e.description(),
            Error::InvalidPacket(..) => "structurally invalid stream packet",
            Error::UnexpectedPacket(..) => "unexpected stream packet",
            Error::IlpError(ref e) => e.description(),
            Error::ConnectionError(..) => "connection error",
            Error::PollError(..) => "poll error",
            Error::SendMoneyError(..) => "error sending money",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref e) => Some(e),
            Error::Utf8Error(ref e) => Some(e),
            Error::FromUtf8Error(ref e) => Some(e),
            Error::InvalidPacket(..) | Error::UnexpectedPacket(..) => None,
            Error::IlpError(ref e) => Some(e),
            Error::ConnectionError(..) | Error::PollError(..) | Error::SendMoneyError(..) => None,
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
impl From<interledger_packet::Error> for Error {
    fn from(e: interledger_packet::Error) -> Error {
        Error::IlpError(e)
    }
}

