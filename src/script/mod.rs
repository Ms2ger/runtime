/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use error::Error;
use js::jsapi::{JS_Init, OnNewGlobalHookOption, CompartmentOptions};
use js::jsapi::{JSAutoRequest, JS_NewGlobalObject, Rooted};
use js::jsapi::{JS_GlobalObjectTraceHook, JSClass};
use js::{JSCLASS_IS_GLOBAL, JSCLASS_RESERVED_SLOTS_MASK};
use js::{JSCLASS_RESERVED_SLOTS_SHIFT, JSCLASS_GLOBAL_SLOT_COUNT};
use js::rust::Runtime;
use libc::c_char;
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

static CLASS: JSClass = JSClass {
    name: b"Global\0" as *const u8 as *const c_char,
    flags: JSCLASS_IS_GLOBAL |
           ((JSCLASS_GLOBAL_SLOT_COUNT & JSCLASS_RESERVED_SLOTS_MASK) <<
            JSCLASS_RESERVED_SLOTS_SHIFT),
    addProperty: None,
    delProperty: None,
    getProperty: None,
    setProperty: None,
    enumerate: None,
    resolve: None,
    convert: None,
    finalize: None,
    call: None,
    hasInstance: None,
    construct: None,
    trace: Some(JS_GlobalObjectTraceHook),
    reserved: [0 as *mut _; 25],
};

pub fn run_script(path: &Path) -> Result<(), Error> {
    let script = try!(load_script(path));
    INIT.call_once(|| {
        unsafe {
            assert!(JS_Init());
        }
    });
    let runtime = Runtime::new();
    let _ar = JSAutoRequest::new(runtime.cx());
    let options = CompartmentOptions::default();
    let global = unsafe {
        JS_NewGlobalObject(runtime.cx(),
                           &CLASS,
                           ptr::null_mut(),
                           OnNewGlobalHookOption::FireOnNewGlobalHook,
                           &options)
    };
    let global = Rooted::new(runtime.cx(), global);
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
