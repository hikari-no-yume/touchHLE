/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Miscellaneous parts of `unistd.h`

use crate::dyld::{export_c_func, FunctionExports};
use crate::fs::GuestPath;
use crate::libc::posix_io::{FileDescriptor, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use crate::mem::ConstPtr;
use crate::Environment;
use std::time::Duration;

#[allow(non_camel_case_types)]
type useconds_t = u32;

const F_OK: i32 = 0;
const R_OK: i32 = 4;
const W_OK: i32 = 2;
const X_OK: i32 = 1;

fn sleep(env: &mut Environment, seconds: u32) -> u32 {
    env.sleep(Duration::from_secs(seconds.into()), true);
    // sleep() returns the amount of time remaining that should have been slept,
    // but wasn't, if the thread was woken up early by a signal.
    // touchHLE never does that currently, so 0 is always correct here.
    0
}

fn usleep(env: &mut Environment, useconds: useconds_t) -> i32 {
    env.sleep(Duration::from_micros(useconds.into()), true);
    0 // success
}

#[allow(non_camel_case_types)]
type pid_t = i32;

fn getpid(_env: &mut Environment) -> pid_t {
    // Not a real value, since touchHLE only simulates a single process.
    // PID 0 would be init, which is a bit unrealistic, so let's go with 1.
    1
}
fn getppid(_env: &mut Environment) -> pid_t {
    // Included just for completeness. Surely no app ever calls this.
    0
}

fn isatty(_env: &mut Environment, fd: FileDescriptor) -> i32 {
    if [STDIN_FILENO, STDOUT_FILENO, STDERR_FILENO].contains(&fd) {
        1
    } else {
        0
    }
}

fn access(env: &mut Environment, path: ConstPtr<u8>, mode: i32) -> i32 {
    let binding = env.mem.cstr_at_utf8(path).unwrap();
    let guest_path = GuestPath::new(&binding);
    let (exists, r, w, x) = env.fs.access(guest_path);
    if mode == F_OK {
        return if exists {
            0
        } else {
            -1
        };
    }
    if mode & R_OK != 0 && !r {
        return -1;
    }
    if mode & W_OK != 0 && !w {
        return -1;
    }
    if mode & X_OK != 0 && !x {
        return -1;
    }
    0
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(sleep(_)),
    export_c_func!(usleep(_)),
    export_c_func!(getpid()),
    export_c_func!(getppid()),
    export_c_func!(isatty(_)),
    export_c_func!(access(_, _)),
];
