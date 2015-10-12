/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin)]
#![feature(plugin_registrar)]
#![feature(rustc_private)]

#![plugin(clippy)]

extern crate clippy;
extern crate js;
extern crate rustc;

use rustc::plugin::Registry;

#[plugin_registrar]
pub fn plugin_registrar(registry: &mut Registry) {
    clippy::plugin_registrar(registry);
}

fn main() {
    println!("Hello, world!");
}
