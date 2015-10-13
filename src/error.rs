/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt::{self, Display, Formatter};

pub enum Error {
    MissingArgument,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::MissingArgument => {
                write!(formatter, "a required argument was omitted")
            }
        }
    }
}
