/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::conversions::FromJSValConvertible;
use js::jsapi::CallArgs;
use js::jsapi::HandleObject;
use js::jsapi::HandleValue;
use js::jsapi::JSAutoCompartment;
use js::jsapi::JSAutoRequest;
use js::jsapi::JSClass;
use js::jsapi::JSContext;
use js::jsapi::JSFunctionSpec;
use js::jsapi::JS_GetClass;
use js::jsapi::JSNativeWrapper;
use js::jsapi::JS_NewObjectWithGivenProto;
use js::jsapi::MutableHandleObject;
use js::jsapi::RootedObject;
use js::jsapi::Value;
use js::JSCLASS_IS_GLOBAL;
use js::JSCLASS_RESERVED_SLOTS_MASK;
use js::JSCLASS_RESERVED_SLOTS_SHIFT;
use js::JSPROP_ENUMERATE;
use libc::c_char;
use script::reflect::{Reflectable, PrototypeID, finalize};
use std::cell::Ref;
use std::cell::RefCell;
use std::ptr;

pub trait ConsoleMessageHandler {
    fn log(&self, s: String);
}

pub struct StdoutHandler;

impl ConsoleMessageHandler for StdoutHandler {
    fn log(&self, s: String) {
        println!("{}", s);
    }
}

pub struct StoringHandler(RefCell<Vec<String>>);

impl StoringHandler {
    fn get(&self) -> Ref<[String]> {
        Ref::map(self.0.borrow(), |x| &**x)
    }
}

impl ConsoleMessageHandler for StoringHandler {
    fn log(&self, s: String) {
        self.0.borrow_mut().push(s);
    }
}

pub struct Console(Box<ConsoleMessageHandler>);

impl Console {
    pub fn new(handler: Box<ConsoleMessageHandler>) -> Console {
        Console(handler)
    }

    pub fn log(&self, message: String) {
        self.0.log(message);
    }
}

static CLASS: JSClass = JSClass {
    name: b"Console\0" as *const u8 as *const c_char,
    // JSCLASS_HAS_RESERVED_SLOTS(1)
    flags: (1 & JSCLASS_RESERVED_SLOTS_MASK) << JSCLASS_RESERVED_SLOTS_SHIFT,
    addProperty: None,
    delProperty: None,
    getProperty: None,
    setProperty: None,
    enumerate: None,
    resolve: None,
    convert: None,
    finalize: Some(finalize::<Console>),
    call: None,
    hasInstance: None,
    construct: None,
    trace: None,
    reserved: [0 as *mut _; 25],
};

static PROTOTYPE_CLASS: JSClass = JSClass {
    name: b"ConsolePrototype\0" as *const u8 as *const c_char,
    flags: 0,
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
    trace: None,
    reserved: [0 as *mut _; 25]
};

const METHODS: &'static [JSFunctionSpec] = &[
    JSFunctionSpec {
        name: b"log\0" as *const u8 as *const c_char,
        call: JSNativeWrapper {op: Some(console_log_native), info: 0 as *const _},
        nargs: 1,
        flags: JSPROP_ENUMERATE as u16,
        selfHostedName: 0 as *const c_char
    },
    JSFunctionSpec {
        name: 0 as *const c_char,
        call: JSNativeWrapper { op: None, info: 0 as *const _ },
        nargs: 0,
        flags: 0,
        selfHostedName: 0 as *const c_char
    }
];

impl Reflectable for Console {
    fn class() -> &'static JSClass {
        &CLASS
    }

    fn prototype_class() -> &'static JSClass {
        &PROTOTYPE_CLASS
    }

    fn methods() -> Option<&'static [JSFunctionSpec]> {
        Some(METHODS)
    }

    fn prototype_index() -> PrototypeID {
        PrototypeID::Console
    }
}


pub unsafe fn create_console(cx: *mut JSContext,
                             scope: HandleObject,
                             handler: Box<ConsoleMessageHandler>,
                             rval: MutableHandleObject)
                             -> Result<(), ()> {
    let console = Box::new(Console::new(handler));
    assert!(!scope.get().is_null());
    assert!(((*JS_GetClass(scope.get())).flags & JSCLASS_IS_GLOBAL) != 0);

    let _ar = JSAutoRequest::new(cx);
    let _ac = JSAutoCompartment::new(cx, scope.get());
    let mut proto = RootedObject::new(cx, ptr::null_mut());
    Console::get_prototype_object(cx, scope, proto.handle_mut());
    assert!(!proto.ptr.is_null());

    rval.set(JS_NewObjectWithGivenProto(cx, &CLASS as *const _, proto.handle()));
    assert!(!rval.get().is_null());

    console.init(rval.get());
    Ok(())
}

unsafe fn to_string(cx: *mut JSContext, value: HandleValue) -> Result<String, ()> {
    String::from_jsval(cx, value, ())
}

unsafe fn console_log(cx: *mut JSContext, args: &CallArgs) -> Result<(), ()> {
    let console = try!(Console::from_value(cx, args.thisv()));
    let message = try!(to_string(cx, args.get(0)));
    (*console).log(message);
    Ok(())
}

unsafe extern "C" fn console_log_native(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
    let args = CallArgs::from_vp(vp, argc);
    console_log(cx, &args).is_ok()
}
