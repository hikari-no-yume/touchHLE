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
use super::{ns_run_loop, ns_thread, NSUInteger};
use crate::frameworks::foundation::ns_string::from_rust_string;
use crate::mem::{ConstVoidPtr, MutVoidPtr};
use crate::objc::{
    autorelease, class_conformsToProtocol, id, msg, msg_class, msg_send, objc_classes, Class,
    ClassExports, NSZonePtr, ObjC, TrivialHostObject, IMP, SEL,
};

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSObject

+ (id)alloc {
    msg![env; this allocWithZone:(MutVoidPtr::null())]
}
+ (id)allocWithZone:(NSZonePtr)_zone { // struct _NSZone*
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

+(())initialize {

}

+ (bool)instancesRespondToSelector:(SEL)selector {
    env.objc.class_has_method(this, selector)
}

+(ConstVoidPtr)instanceMethodForSelector:(SEL)selector {
    match ObjC::lookup_method(env, this, selector).map(|x| x.imp) {
        Some(IMP::Guest(g)) => ConstVoidPtr::from_bits(g.addr_with_thumb_bit()),
        Some(_) => todo!(),
        None => panic!("Unable to lookup method {:?} in class {:?}", selector.as_str(&env.mem), this)
    }
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
- (NSUInteger)retainCount {
    env.objc.get_refcount(this)
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

// TODO: description and debugDescription (both the instance and class method).
// This is not hard to add, but before adding a fallback implementation of it,
// we should make sure all the Foundation classes' overrides of it are there,
// to prevent weird behavior.
// TODO: localized description methods also? (not sure if NSObject has them)

-(id)description {
    let class = msg![env; this class];
    let desc = from_rust_string(env, format!(
        "<{}: {:#x}>", env.objc.get_class_name(class), this.to_bits()
    ));
    autorelease(env, desc)
}

// Helper for NSCopying
- (id)copy {
    msg![env; this copyWithZone:(MutVoidPtr::null())]
}

// Helper for NSMutableCopying
- (id)mutableCopy {
    msg![env; this mutableCopyWithZone:(MutVoidPtr::null())]
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

- (id)valueForKey:(id)key {
    let key = to_rust_string(env, key);

    let class = msg![env; this class];

    if let Some(sel) = env.objc.lookup_selector(key.as_ref()) {
        if let Some(mt) = ObjC::lookup_method(env, class, sel) {
            return match mt.type_[0] {
                b'@' => msg_send(env, (this, sel)),
                b'i' => {
                    let ret: i32 = msg_send(env, (this, sel));
                    msg_class![env; NSNumber numberWithInt: ret]
                },
                t => todo!("Unsupported type: {}", t as char)
            }
        }
    }

    unimplemented!("TODO: object {:?} does not have simple getter method for {}, use fallback", this, key);
}

- (bool)respondsToSelector:(SEL)selector {
    let class = msg![env; this class];
    env.objc.class_has_method(class, selector)
}

- (id)performSelector:(SEL)sel {
    assert!(!sel.is_null());
    msg_send(env, (this, sel))
}

- (id)performSelector:(SEL)sel
           withObject:(id)o1 {
    assert!(!sel.is_null());
    msg_send(env, (this, sel, o1))
}

- (id)performSelector:(SEL)sel
           withObject:(id)o1
           withObject:(id)o2 {
    assert!(!sel.is_null());
    msg_send(env, (this, sel, o1, o2))
}

- (ConstVoidPtr)methodForSelector:(SEL)selector {
    let isa = ObjC::read_isa(this, &env.mem);
    match ObjC::lookup_method(env, isa, selector).map(|x| x.imp) {
        Some(IMP::Guest(g)) => ConstVoidPtr::from_bits(g.addr_with_thumb_bit()),
        Some(_) => todo!(),
        None => panic!("Unable to lookup method {:?} in class {:?}", selector.as_str(&env.mem), isa)
    }
}

- (bool)conformsToProtocol:(id)protocol {
    let class = msg![env; this class];
    class_conformsToProtocol(env, class, protocol)
}

- (())performSelector:(SEL)selector
               onThread:(id)thread
             withObject:(id)arg
          waitUntilDone:(bool)wait {
    if wait {
        log!("performSelector with true waitUntilDone, let's hope it wasn't actually needed");
    }
    let run_loop = ns_thread::get_run_loop(env, thread);
    ns_run_loop::schedule_invocation(env, run_loop, this, selector, arg);
}

@end

};