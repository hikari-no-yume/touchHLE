/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `printf` function family.

use crate::abi::VAList;
use crate::dyld::{export_c_func, FunctionExports};
use crate::mem::{ConstPtr, MutPtr};
use crate::Environment;
use std::io::Write;

fn printf_inner(env: &mut Environment, format: ConstPtr<u8>, mut args: VAList) -> Vec<u8> {
    log_dbg!(
        "Processing format string {:?}",
        env.mem.cstr_at_utf8(format)
    );

    let mut res = Vec::<u8>::new();

    let mut current_format = format;

    loop {
        let c = env.mem.read(current_format);
        current_format += 1;

        if c == b'\0' {
            break;
        }
        if c != b'%' {
            res.push(c);
            continue;
        }

        let pad_char = if env.mem.read(current_format) == b'0' {
            current_format += 1;
            '0'
        } else {
            ' '
        };
        let pad_width = {
            let mut pad_width = 0;
            while let c @ b'0'..=b'9' = env.mem.read(current_format) {
                pad_width = pad_width * 10 + (c - b'0') as usize;
                current_format += 1;
            }
            pad_width
        };

        let specifier = env.mem.read(current_format);
        current_format += 1;

        assert!(specifier != b'\0');
        if specifier == b'%' {
            res.push(b'%');
            continue;
        }

        match specifier {
            b's' => {
                let c_string: ConstPtr<u8> = args.next(env);
                assert!(pad_char == ' ' && pad_width == 0); // TODO
                res.extend_from_slice(env.mem.cstr_at(c_string));
            }
            b'd' | b'i' => {
                let int: i32 = args.next(env);
                // TODO: avoid copy?
                if pad_width > 0 {
                    if pad_char == '0' {
                        res.extend_from_slice(format!("{:01$}", int, pad_width).as_bytes());
                    } else {
                        res.extend_from_slice(format!("{:1$}", int, pad_width).as_bytes());
                    }
                } else {
                    res.extend_from_slice(format!("{}", int).as_bytes());
                }
            }
            b'f' => {
                let float: f64 = args.next(env);
                // TODO: avoid copy?
                if pad_width > 0 {
                    if pad_char == '0' {
                        res.extend_from_slice(format!("{:01$}", float, pad_width).as_bytes());
                    } else {
                        res.extend_from_slice(format!("{:1$}", float, pad_width).as_bytes());
                    }
                } else {
                    res.extend_from_slice(format!("{}", float).as_bytes());
                }
            }
            // TODO: more specifiers
            _ => unimplemented!("Format character '{}'", specifier as char),
        }
    }

    log_dbg!("=> {:?}", std::str::from_utf8(&res));

    res
}

fn sprintf(env: &mut Environment, dest: MutPtr<u8>, format: ConstPtr<u8>, args: VAList) -> i32 {
    let res = printf_inner(env, format, args);

    log_dbg!("sprintf({:?}, {:?}, ...)", dest, format);

    let dest_slice = env
        .mem
        .bytes_at_mut(dest, (res.len() + 1).try_into().unwrap());
    for (i, &byte) in res.iter().chain(b"\0".iter()).enumerate() {
        dest_slice[i] = byte;
    }

    res.len().try_into().unwrap()
}

fn printf(env: &mut Environment, format: ConstPtr<u8>, args: VAList) -> i32 {
    let res = printf_inner(env, format, args);
    // TODO: I/O error handling
    let _ = std::io::stdout().write_all(&res);
    res.len().try_into().unwrap()
}

// TODO: more printf variants

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(sprintf(_, _, _)),
    export_c_func!(printf(_, _)),
];
