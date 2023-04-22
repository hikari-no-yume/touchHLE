/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSFileManager` etc.

use super::{ns_array, ns_string, NSUInteger};
use crate::dyld::{export_c_func, FunctionExports};
use crate::fs::GuestPath;
use crate::objc::{autorelease, id, msg, msg_class, nil, objc_classes, release, ClassExports};
use crate::Environment;

type NSSearchPathDirectory = NSUInteger;
const NSDocumentDirectory: NSSearchPathDirectory = 9;

type NSSearchPathDomainMask = NSUInteger;
const NSUserDomainMask: NSSearchPathDomainMask = 1;

fn NSSearchPathForDirectoriesInDomains(
    env: &mut Environment,
    directory: NSSearchPathDirectory,
    domain_mask: NSSearchPathDomainMask,
    expand_tilde: bool,
) -> id {
    // TODO: other cases not implemented
    assert!(directory == NSDocumentDirectory);
    assert!(domain_mask == NSUserDomainMask);
    assert!(expand_tilde);

    let dir = env.fs.home_directory().join("Documents");
    let dir = ns_string::from_rust_string(env, String::from(dir));
    let dir_list = ns_array::from_vec(env, vec![dir]);
    autorelease(env, dir_list)
}

fn NSHomeDirectory(env: &mut Environment) -> id {
    let dir = env.fs.home_directory();
    let dir = ns_string::from_rust_string(env, String::from(dir.as_str()));
    autorelease(env, dir)
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(NSHomeDirectory()),
    export_c_func!(NSSearchPathForDirectoriesInDomains(_, _, _)),
];

#[derive(Default)]
pub struct State {
    default_manager: Option<id>,
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSFileManager: NSObject

+ (id)defaultManager {
    if let Some(existing) = env.framework_state.foundation.ns_file_manager.default_manager {
        existing
    } else {
        let new: id = msg![env; this new];
        env.framework_state.foundation.ns_file_manager.default_manager = Some(new);
        autorelease(env, new)
    }
}

- (bool)fileExistsAtPath:(id)path { // NSString*
    let path = ns_string::to_rust_string(env, path); // TODO: avoid copy
    // fileExistsAtPath: will return true for directories, hence Fs::exists()
    // rather than Fs::is_file() is appropriate.
    let res = env.fs.exists(GuestPath::new(&path));
    log_dbg!("fileExistsAtPath:{:?} => {}", path, res);
    res
}

- (bool)createFileAtPath:(id)path // NSString*
                contents:(id)data // NSData*
              attributes:(id)attributes { // NSDictionary*
    assert!(attributes == nil); // TODO

    let path_str = ns_string::to_rust_string(env, path); // TODO: avoid copy
    // createFileAtPath: returns true if there's already a file at a given path.
    // If there's a directory, that's an error, though.
    if env.fs.is_file(GuestPath::new(&path_str)) {
        return true;
    }

    if data == nil {
        let empty: id = msg_class![env; NSData new];
        let res: bool = msg![env; empty writeToFile:path atomically:false];
        release(env, empty);
        res
    } else {
        msg![env; data writeToFile:path atomically:false]
    }
}

@end

};
