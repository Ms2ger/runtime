/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod console;
mod global;
mod reflect;

use error::Error;
use js::jsapi::{JS_Init, JSAutoRequest, Rooted};
use js::rust::Runtime;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr;
use std::sync::{Once, ONCE_INIT};

static INIT: Once = ONCE_INIT;

fn load_script(path: &Path) -> Result<String, Error> {
    let mut file = try!(File::open(path));
    let mut buffer = vec![];
    try!(file.read_to_end(&mut buffer));
    let script = try!(String::from_utf8(buffer));
    Ok(script)
}

pub fn run_script(path: &Path) -> Result<(), Error> {
    let script = try!(load_script(path));
    INIT.call_once(|| {
        unsafe {
            assert!(JS_Init());
        }
    });

    let runtime = Runtime::new();
    let _ar = JSAutoRequest::new(runtime.cx());
    let mut global = Rooted::new(runtime.cx(), ptr::null_mut());
    unsafe { global::create(runtime.cx(), global.handle_mut()) };
    assert!(!global.ptr.is_null());

    let path_string = path.to_string_lossy().into_owned();
    try!(runtime.evaluate_script(global.handle(), script, path_string, 0));
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

#[test]
fn running_tests() {
    match run_script(Path::new("test-files/success.js")) {
        Ok(()) => (),
        Err(error) => panic!("Unexpected error: {}", error),
    }
}

#[test]
fn reference_error() {
    match run_script(Path::new("test-files/reference-error.js")) {
        Err(Error::Error) => (),
        Err(error) => panic!("Unexpected error: {}", error),
        Ok(()) => panic!("Unexpected ok"),
    }
}

#[test]
fn syntax_error() {
    match run_script(Path::new("test-files/syntax-error.js")) {
        Err(Error::Error) => (),
        Err(error) => panic!("Unexpected error: {}", error),
        Ok(()) => panic!("Unexpected ok"),
    }
}
