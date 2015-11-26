/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::jsapi::CallArgs;
use js::jsapi::CompartmentOptions;
use js::jsapi::JSAutoCompartment;
use js::jsapi::JSClass;
use js::jsapi::JSContext;
use js::jsapi::JS_FireOnNewGlobalObject;
use js::jsapi::JS_GlobalObjectTraceHook;
use js::jsapi::JS_InitStandardClasses;
use js::jsapi::JSNativeWrapper;
use js::jsapi::JS_NewGlobalObject;
use js::jsapi::JSObject;
use js::jsapi::JSPropertySpec;
use js::jsapi::JS_SetPrototype;
use js::jsapi::JSTraceOp;
use js::jsapi::JSVersion;
use js::jsapi::MutableHandleObject;
use js::jsapi::OnNewGlobalHookOption;
use js::jsapi::RootedObject;
use js::jsapi::Value;
use js::JSCLASS_GLOBAL_SLOT_COUNT;
use js::JSCLASS_IS_GLOBAL;
use js::JSCLASS_RESERVED_SLOTS_MASK;
use js::JSCLASS_RESERVED_SLOTS_SHIFT;
use js::JSPROP_ENUMERATE;
use js::JSPROP_SHARED;
use js::jsval::ObjectValue;
use libc::c_char;
use script::console;
use script::reflect::{Reflectable, PrototypeID, finalize, initialize_global};
use std::ptr;
use std::env;
use std::process;
use js::jsapi::JSFunctionSpec;
use js::conversions::FromJSValConvertible;

pub struct Global(usize);

impl Global {
    fn launch_browser(&self, url: String) {
        let path_to_self = env::var_os("SERVO_PATH").unwrap();
        let mut child_process = process::Command::new(path_to_self);
        child_process.arg(url);
        let _ = child_process.spawn().unwrap();
    }
}

static CLASS: JSClass = JSClass {
    name: b"Global\0" as *const u8 as *const c_char,
    flags: JSCLASS_IS_GLOBAL |
           (((JSCLASS_GLOBAL_SLOT_COUNT + 1) & JSCLASS_RESERVED_SLOTS_MASK) <<
            JSCLASS_RESERVED_SLOTS_SHIFT),
    addProperty: None,
    delProperty: None,
    getProperty: None,
    setProperty: None,
    enumerate: None,
    resolve: None,
    convert: None,
    finalize: Some(finalize::<Global>),
    call: None,
    hasInstance: None,
    construct: None,
    trace: Some(JS_GlobalObjectTraceHook),
    reserved: [0 as *mut _; 25],
};

static PROTOTYPE_CLASS: JSClass = JSClass {
    name: b"GlobalPrototype\0" as *const u8 as *const c_char,
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
    reserved: [0 as *mut _; 25],
};

const ATTRIBUTES: &'static [JSPropertySpec] = &[
    JSPropertySpec {
        name: b"console\0" as *const u8 as *const c_char,
        flags: ((JSPROP_SHARED | JSPROP_ENUMERATE) & 0xFF) as u8,
        getter: JSNativeWrapper { op: Some(get_console_native), info: 0 as *const _ },
        setter: JSNativeWrapper { op: None, info: 0 as *const _ }
    },
    JSPropertySpec {
        name: 0 as *const c_char,
        flags: 0,
        getter: JSNativeWrapper { op: None, info: 0 as *const _ },
        setter: JSNativeWrapper { op: None, info: 0 as *const _ }
    }
];

const METHODS: &'static [JSFunctionSpec] = &[
    JSFunctionSpec {
        name: b"launchBrowser\0" as *const u8 as *const c_char,
        call: JSNativeWrapper {op: Some(launch_browser_native), info: 0 as *const _},
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

impl Reflectable for Global {
    fn class() -> &'static JSClass {
        &CLASS
    }

    fn prototype_class() -> &'static JSClass {
        &PROTOTYPE_CLASS
    }

    fn attributes() -> Option<&'static [JSPropertySpec]> {
        Some(ATTRIBUTES)
    }

    fn methods() -> Option<&'static [JSFunctionSpec]> {
        Some(METHODS)
    }

    fn prototype_index() -> PrototypeID {
        PrototypeID::Global
    }
}

unsafe fn get_console(cx: *mut JSContext, args: &CallArgs) -> Result<(), ()> {
    let thisv = args.thisv();
    let scope = RootedObject::new(cx, thisv.to_object());
    let mut rval = RootedObject::new(cx, ptr::null_mut());
    try!(console::create_console(cx,
                                 scope.handle(),
                                 Box::new(console::StdoutHandler),
                                 rval.handle_mut()));
    args.rval().set(ObjectValue(&*rval.ptr));
    Ok(())
}

unsafe extern "C" fn get_console_native(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
    let args = CallArgs::from_vp(vp, argc);
    get_console(cx, &args).is_ok()
}

unsafe fn launch_browser(cx: *mut JSContext, args: &CallArgs) -> Result<(), ()> {
    let global = try!(Global::from_value(cx, args.thisv()));
    let url = try!(String::from_jsval(cx, args.get(0), ()));
    (*global).launch_browser(url);
    Ok(())
}

unsafe extern "C" fn launch_browser_native(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
    let args = CallArgs::from_vp(vp, argc);
    launch_browser(cx, &args).is_ok()
}

/// Create a DOM global object with the given class.
pub fn create_dom_global(cx: *mut JSContext,
                         class: &'static JSClass,
                         global: Box<Global>,
                         trace: JSTraceOp)
                         -> *mut JSObject {
    unsafe {
        let mut options = CompartmentOptions::default();
        options.version_ = JSVersion::JSVERSION_ECMA_5;
        options.traceGlobal_ = trace;

        let obj =
            RootedObject::new(cx,
                              JS_NewGlobalObject(cx,
                                                 class,
                                                 ptr::null_mut(),
                                                 OnNewGlobalHookOption::DontFireOnNewGlobalHook,
                                                 &options));
        assert!(!obj.ptr.is_null());
        let _ac = JSAutoCompartment::new(cx, obj.ptr);
        global.init(obj.ptr);
        JS_InitStandardClasses(cx, obj.handle());
        initialize_global(obj.ptr);
        JS_FireOnNewGlobalObject(cx, obj.handle());
        obj.ptr
    }
}


pub unsafe fn create(cx: *mut JSContext, rval: MutableHandleObject) {
    rval.set(create_dom_global(cx, &CLASS, Box::new(Global(0)), None));
    let _ac = JSAutoCompartment::new(cx, rval.handle().get());
    let mut proto = RootedObject::new(cx, ptr::null_mut());
    Global::get_prototype_object(cx, rval.handle(), proto.handle_mut());
    assert!(JS_SetPrototype(cx, rval.handle(), proto.handle()));
}
