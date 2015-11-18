/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use js::error::throw_type_error;
use js::jsapi::HandleObject;
use js::jsapi::HandleValue;
use js::jsapi::JSClass;
use js::jsapi::JSContext;
use js::jsapi::JSFreeOp;
use js::jsapi::JSFunctionSpec;
use js::jsapi::JS_GetClass;
use js::jsapi::JS_GetObjectPrototype;
use js::jsapi::JS_GetReservedSlot;
use js::jsapi::JS_NewObjectWithUniqueType;
use js::jsapi::JSObject;
use js::jsapi::JSPropertySpec;
use js::jsapi::JS_SetReservedSlot;
use js::jsapi::MutableHandleObject;
use js::jsapi::RootedObject;
use js::jsval::PrivateValue;
use js::rust::define_methods;
use js::rust::define_properties;

const DOM_OBJECT_SLOT: u32 = 0;

pub trait Reflectable: Sized {
    fn class() -> &'static JSClass;
    fn prototype_class() -> &'static JSClass;

    unsafe fn init(self: Box<Self>, object: *mut JSObject) {
        JS_SetReservedSlot(object,
                           DOM_OBJECT_SLOT,
                           PrivateValue(Box::into_raw(self) as *const _));
    }

    unsafe fn is(object: *mut JSObject) -> bool {
        JS_GetClass(object) == Self::class()
    }

    unsafe fn from_reflector(object: *mut JSObject) -> *const Self {
        assert!(Self::is(object));
        let slot = JS_GetReservedSlot(object, DOM_OBJECT_SLOT);
        slot.to_private() as *const _
    }

    unsafe fn from_value(cx: *mut JSContext, v: HandleValue) -> Result<*const Self, ()> {
        if !v.is_object() {
            throw_type_error(cx, "Value is not an object");
            return Err(());
        }

        let object = v.to_object();
        if !Self::is(object) {
            throw_type_error(cx, "Value is not an object");
            return Err(());
        }

        Ok(Self::from_reflector(object))
    }

    fn methods() -> Option<&'static [JSFunctionSpec]> {
        None
    }

    fn attributes() -> Option<&'static [JSPropertySpec]> {
        None
    }

    unsafe fn get_parent_proto(cx: *mut JSContext, global: HandleObject) -> *mut JSObject {
        let parent_proto = JS_GetObjectPrototype(cx, global);
        assert!(!parent_proto.is_null());
        parent_proto
    }

    unsafe fn create_interface_prototype_object(cx: *mut JSContext,
                                                global: HandleObject,
                                                rval: MutableHandleObject) {
        assert!(rval.get().is_null());

        let parent_proto = RootedObject::new(cx, Self::get_parent_proto(cx, global));

        rval.set(JS_NewObjectWithUniqueType(cx, Self::prototype_class(), parent_proto.handle()));
        assert!(!rval.get().is_null());

        if let Some(methods) = Self::methods() {
            define_methods(cx, rval.handle(), methods).unwrap();
        }

        if let Some(attributes) = Self::attributes() {
            define_properties(cx, rval.handle(), attributes).unwrap();
        }
    }

    unsafe fn get_prototype_object(cx: *mut JSContext,
                                   global: HandleObject,
                                   rval: MutableHandleObject) {
        // TODO: cache
        Self::create_interface_prototype_object(cx, global, rval);
        assert!(!rval.get().is_null());
    }
}

pub unsafe extern "C" fn finalize<T: Reflectable>(_fop: *mut JSFreeOp, object: *mut JSObject) {
    let this = T::from_reflector(object);
    let _ = Box::from_raw(this as *mut T);
}
