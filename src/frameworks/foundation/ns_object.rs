/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSObject`, the root of most class hierarchies in Objective-C.
//!
//! Resources:
//! - Apple's [Advanced Memory Management Programming Guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/MemoryMgmt/Articles/MemoryMgmt.html)
//!   explains how reference counting works. Note that we are interested in what
//!   it calls "manual retain-release", not ARC.
//! - Apple's [Key-Value Coding Programming Guide](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/KeyValueCoding/SearchImplementation.html)
//!   explains the algorithm `setValue:forKey:` should follow.
//!
//! See also: [crate::objc], especially the `objects` module.

use super::ns_string::to_rust_string;
use super::NSUInteger;
use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::MutVoidPtr;
use crate::objc::{
    id, msg, msg_class, msg_send, objc_classes, Class, ClassExports, ObjC, TrivialHostObject, SEL,
};
use crate::Environment;

pub(super) fn NSClassFromString(
    env: &mut Environment,
    _class_name: id, // NSString
) -> Class {
    // TODO: actually implement this
    println!("Warning: NSClassFromString not implemented, app will crash.");
    return msg_class![env; NSObject alloc];
}

pub(super) fn NSLog(
    _env: &mut Environment,
    _string: id, // NSString
) {
    // TODO: actually implement this
    println!("Warning: NSLog not implemented, the app is logging something and we can't see it.");
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(NSClassFromString(_)),
    export_c_func!(NSLog(_)),
];

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSObject

+ (id)alloc {
    msg![env; this allocWithZone:(MutVoidPtr::null())]
}
+ (id)allocWithZone:(MutVoidPtr)_zone { // struct _NSZone*
    log_dbg!("[{:?} allocWithZone:]", this);
    env.objc.alloc_object(this, Box::new(TrivialHostObject), &mut env.mem)
}

+ (id)new {
    let new_object: id = msg![env; this alloc];
    msg![env; new_object init]
}

+ (Class)class {
    this
}

// See the instance method section for the normal versions of these.
+ (id)retain {
    this // classes are not refcounted
}
+ (())release {
    // classes are not refcounted
}
+ (())autorelease {
    // classes are not refcounted
}

- (id)init {
    this
}


- (id)retain {
    log_dbg!("[{:?} retain]", this);
    env.objc.increment_refcount(this);
    this
}
- (())release {
    log_dbg!("[{:?} release]", this);
    if env.objc.decrement_refcount(this) {
        () = msg![env; this dealloc];
    }
}
- (id)autorelease {
    () = msg_class![env; NSAutoreleasePool addObject:this];
    this
}

- (())dealloc {
    log_dbg!("[{:?} dealloc]", this);
    env.objc.dealloc_object(this, &mut env.mem)
}

- (Class)class {
    ObjC::read_isa(this, &env.mem)
}
- (bool)isMemberOfClass:(Class)class {
    let this_class: Class = msg![env; this class];
    class == this_class
}
- (bool)isKindOfClass:(Class)class {
    let this_class: Class = msg![env; this class];
    env.objc.class_is_subclass_of(this_class, class)
}

- (NSUInteger)hash {
    this.to_bits()
}
- (bool)isEqual:(id)other {
    this == other
}

- (bool)respondsToSelector:(SEL)_selector { // SEL
    // TODO: actually implement this
    true
}

// Helper for NSCopying
- (id)copy {
    msg![env; this copyWithZone:(MutVoidPtr::null())]
}


// NSKeyValueCoding
- (())setValue:(id)value
       forKey:(id)key { // NSString*
    let key = to_rust_string(env, key); // TODO: avoid copy?
    assert!(key.is_ascii()); // TODO: do we have to handle non-ASCII keys?

    let class = msg![env; this class];

    if let Some(sel) = env.objc.lookup_selector(&format!(
        "set{}{}:",
        key.as_bytes()[0].to_ascii_uppercase() as char,
        &key[1..],
    )) {
        if env.objc.class_has_method(class, sel) {
            return msg_send(env, (this, sel, value));
        }
    }

    if let Some(sel) = env.objc.lookup_selector(&format!(
        "_set{}{}:",
        key.as_bytes()[0].to_ascii_uppercase() as char,
        &key[1..],
    )) {
        if env.objc.class_has_method(class, sel) {
            return msg_send(env, (this, sel, value));
        }
    }

    unimplemented!("TODO: object {:?} does not have simple setter method for {}, use fallback", this, key);
}

@end

};
