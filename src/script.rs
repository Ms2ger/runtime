/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn load_script(path: &Path) -> Result<String, Error> {
    let mut file = try!(File::open(path));
    let mut buffer = vec![];
    try!(file.read_to_end(&mut buffer));
    let script = try!(String::from_utf8(buffer));
    Ok(script)
}

pub fn run_script(path: &Path) -> Result<(), Error> {
    let _script = try!(load_script(path));
    Ok(())
}

#[test]
fn missing_file() {
    match load_script(Path::new("test-files/missing.js")) {
        Err(Error::IO(_)) => (),
        Err(error) => panic!("Unexpected error: {}", error),
        Ok(_) => panic!("Unexpected ok"),
    }
}

#[test]
fn non_utf8_file() {
    match load_script(Path::new("test-files/non-utf8.js")) {
        Err(Error::InvalidString(_)) => (),
        Err(error) => panic!("Unexpected error: {}", error),
        Ok(_) => panic!("Unexpected ok"),
    }
}
