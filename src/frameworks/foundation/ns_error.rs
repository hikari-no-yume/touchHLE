/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */


use crate::dyld::{ConstantExports, HostConstant};
use crate::frameworks::foundation::NSInteger;
use crate::objc::{ClassExports, HostObject, id, retain, release, nil, NSZonePtr};
use crate::objc_classes;

struct ErrorHostObject {
    domain: id,
    code: NSInteger,
    user_info: id,
}
impl HostObject for ErrorHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSError: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(ErrorHostObject {
        domain: nil,
        code: 0,
        user_info: nil
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

- (id)initWithDomain:(id)domain
                code:(NSInteger)code
            userInfo:(id)user_info {
    retain(env, domain);
    retain(env, user_info);
    let host_obj = env.objc.borrow_mut::<ErrorHostObject>(this);
    host_obj.domain = domain;
    host_obj.code = code;
    host_obj.user_info = user_info;
    this
}

- (())dealloc {
    let &ErrorHostObject{domain, user_info, ..} = env.objc.borrow(this);
    release(env, domain);
    release(env, user_info);

    env.objc.dealloc_object(this, &mut env.mem);
}


@end

};

pub const CONSTANTS: ConstantExports = &[
    (
        "_NSLocalizedDescriptionKey",
        HostConstant::NSString("NSLocalizedDescriptionKey"),
    ),
];