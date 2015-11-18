/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![feature(plugin_registrar)]
#![feature(rustc_private)]

#![plugin(clippy)]

extern crate clippy;
extern crate env_logger;
extern crate js;
extern crate libc;
extern crate rustc;

mod error;
mod script;

use error::Error;
use rustc::plugin::Registry;
use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process;

#[plugin_registrar]
pub fn plugin_registrar(registry: &mut Registry) {
    clippy::plugin_registrar(registry);
}

fn do_main(path: Option<OsString>) -> Result<(), Error> {
    let path = try!(path.ok_or(Error::MissingArgument));
    script::run_script(Path::new(&path))
}

fn main() {
    env_logger::init().unwrap();
    match do_main(env::args_os().nth(1)) {
        Ok(()) => println!("Hello, world!"),
        Err(error) => {
            println!("Finished unsuccessfully: {}.", error);
            process::exit(1);
        }
    }
}

#[test]
fn missing_argument() {
    match do_main(None) {
        Err(Error::MissingArgument) => (),
        Err(error) => panic!("Unexpected error: {}", error),
        Ok(()) => panic!("Unexpected ok"),
    }
}
