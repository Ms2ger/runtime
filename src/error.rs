/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt::{self, Display, Formatter};
use std::io;
use std::string::FromUtf8Error;

pub enum Error {
    Error,
    InvalidString(FromUtf8Error),
    IO(io::Error),
    MissingArgument,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::Error => {
                write!(formatter, "an unspecified error occurred")
            }
            Error::InvalidString(ref error) => {
                write!(formatter,
                       "an error occurred decoding a string ({:?})",
                       error)
            }
            Error::IO(ref error) => {
                write!(formatter, "an input/output error occurred ({:?})", error)
            }
            Error::MissingArgument => {
                write!(formatter, "a required argument was omitted")
            }
        }
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Error {
        Error::Error
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Error {
        Error::InvalidString(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}
