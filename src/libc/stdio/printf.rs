/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `printf` function family. The implementation is also used by `NSLog` etc.

use crate::abi::{DotDotDot, VaList};
use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::foundation::{ns_string, unichar};
use crate::libc::posix_io::{STDERR_FILENO, STDOUT_FILENO};
use crate::libc::stdio::FILE;
use crate::mem::{ConstPtr, GuestUSize, Mem, MutPtr, MutVoidPtr};
use crate::objc::{id, msg};
use crate::Environment;
use std::io::Write;

const INTEGER_SPECIFIERS: [u8; 6] = [b'd', b'i', b'o', b'u', b'x', b'X'];
const FLOAT_SPECIFIERS: [u8; 3] = [b'f', b'e', b'g'];

/// String formatting implementation for `printf` and `NSLog` function families.
///
/// `NS_LOG` is [true] for the `NSLog` format string type, or [false] for the
/// `printf` format string type.
///
/// `get_format_char` is a callback that returns the byte at a given index in
/// the format string, or `'\0'` if the index is one past the last byte.
pub fn printf_inner<const NS_LOG: bool, F: Fn(&Mem, GuestUSize) -> u8>(
    env: &mut Environment,
    get_format_char: F,
    mut args: VaList,
) -> Vec<u8> {
    let mut res = Vec::<u8>::new();

    let mut format_char_idx = 0;

    loop {
        let c = get_format_char(&env.mem, format_char_idx);
        format_char_idx += 1;

        if c == b'\0' {
            break;
        }
        if c != b'%' {
            res.push(c);
            continue;
        }

        let pad_char = if get_format_char(&env.mem, format_char_idx) == b'0' {
            format_char_idx += 1;
            '0'
        } else {
            ' '
        };

        let pad_width = if get_format_char(&env.mem, format_char_idx) == b'*' {
            let pad_width = args.next::<i32>(env);
            assert!(pad_width >= 0); // TODO: Implement right-padding
            format_char_idx += 1;
            pad_width
        } else {
            let mut pad_width: i32 = 0;
            while let c @ b'0'..=b'9' = get_format_char(&env.mem, format_char_idx) {
                pad_width = pad_width * 10 + (c - b'0') as i32;
                format_char_idx += 1;
            }
            pad_width
        };

        let precision = if get_format_char(&env.mem, format_char_idx) == b'.' {
            format_char_idx += 1;
            let mut precision = 0;
            while let c @ b'0'..=b'9' = get_format_char(&env.mem, format_char_idx) {
                precision = precision * 10 + (c - b'0') as usize;
                format_char_idx += 1;
            }
            Some(precision)
        } else {
            None
        };

        let length_modifier = if get_format_char(&env.mem, format_char_idx) == b'l' {
            format_char_idx += 1;
            Some(b'l')
        } else {
            None
        };

        let specifier = get_format_char(&env.mem, format_char_idx);
        format_char_idx += 1;

        assert!(specifier != b'\0');
        if specifier == b'%' {
            res.push(b'%');
            continue;
        }

        if precision.is_some() {
            assert!(
                INTEGER_SPECIFIERS.contains(&specifier) || FLOAT_SPECIFIERS.contains(&specifier)
            )
        }

        match specifier {
            // Integer specifiers
            b'c' => {
                // TODO: support length modifier
                assert!(length_modifier.is_none());
                let c: u8 = args.next(env);
                assert!(pad_char == ' ' && pad_width == 0); // TODO
                res.push(c);
            }
            // Apple extension? Seemingly works in both NSLog and printf.
            b'C' => {
                assert!(length_modifier.is_none());
                let c: unichar = args.next(env);
                // TODO
                assert!(pad_char == ' ' && pad_width == 0);
                // This will panic if it's a surrogate! This isn't good if
                // targeting UTF-16 ([NSString stringWithFormat:] etc).
                let c = char::from_u32(c.into()).unwrap();
                write!(&mut res, "{}", c).unwrap();
            }
            b's' => {
                // TODO: support length modifier
                assert!(length_modifier.is_none());
                let c_string: ConstPtr<u8> = args.next(env);
                assert!(pad_char == ' ' && pad_width == 0); // TODO
                if !c_string.is_null() {
                    res.extend_from_slice(env.mem.cstr_at(c_string));
                } else {
                    res.extend_from_slice("(null)".as_bytes());
                }
            }
            b'd' | b'i' | b'u' => {
                // Note: on 32-bit system int and long are i32,
                // so length_modifier is ignored
                let int: i64 = if specifier == b'u' {
                    let uint: u32 = args.next(env);
                    uint.into()
                } else {
                    let int: i32 = args.next(env);
                    int.into()
                };

                let int_with_precision = if precision.is_some_and(|value| value > 0) {
                    format!("{:01$}", int, precision.unwrap())
                } else {
                    format!("{}", int)
                };

                if pad_width > 0 {
                    let pad_width = pad_width as usize;
                    if pad_char == '0' && precision.is_none() {
                        write!(&mut res, "{:0>1$}", int_with_precision, pad_width).unwrap();
                    } else {
                        write!(&mut res, "{:>1$}", int_with_precision, pad_width).unwrap();
                    }
                } else {
                    res.extend_from_slice(int_with_precision.as_bytes());
                }
            }
            b'@' if NS_LOG => {
                assert!(length_modifier.is_none());
                let object: id = args.next(env);
                // TODO: use localized description if available?
                let description: id = msg![env; object description];
                // TODO: avoid copy
                // TODO: what if the description isn't valid UTF-16?
                let description = ns_string::to_rust_string(env, description);
                write!(&mut res, "{}", description).unwrap();
            }
            b'x' => {
                // Note: on 32-bit system unsigned int and unsigned long
                // are u32, so length_modifier is ignored
                let uint: u32 = args.next(env);
                res.extend_from_slice(format!("{:x}", uint).as_bytes());
            }
            b'X' => {
                // Note: on 32-bit system unsigned int and unsigned long
                // are u32, so length_modifier is ignored
                let uint: u32 = args.next(env);
                res.extend_from_slice(format!("{:X}", uint).as_bytes());
            }
            b'p' => {
                assert!(length_modifier.is_none());
                let ptr: MutVoidPtr = args.next(env);
                res.extend_from_slice(format!("{:?}", ptr).as_bytes());
            }
            // Float specifiers
            b'f' => {
                let float: f64 = args.next(env);
                let pad_width = pad_width as usize;
                let precision = precision.unwrap_or(6);
                if pad_char == '0' {
                    res.extend_from_slice(
                        format!("{:01$.2$}", float, pad_width, precision).as_bytes(),
                    );
                } else {
                    res.extend_from_slice(
                        format!("{:1$.2$}", float, pad_width, precision).as_bytes(),
                    );
                }
            }
            b'e' => {
                let float: f64 = args.next(env);
                let pad_width = pad_width as usize;
                let precision = precision.unwrap_or(6);

                let exponent = float.log10().floor();
                let mantissa = float / 10f64.powf(exponent);
                let sign = if float.is_sign_positive() { '+' } else { '-' };
                // Format without padding
                let float_exp_notation =
                    format!("{0:.1$}e{2}{3:02}", mantissa, precision, sign, exponent);

                if pad_char == '0' {
                    res.extend_from_slice(
                        format!("{:0>1$}", float_exp_notation, pad_width).as_bytes(),
                    );
                } else {
                    res.extend_from_slice(
                        format!("{:>1$}", float_exp_notation, pad_width).as_bytes(),
                    );
                }
            }
            b'g' => {
                let float: f64 = args.next(env);

                let formatted_f = {
                    // Precision in %g means max number of decimal digits in
                    // the mantissa. For that, we first calculate the length
                    // of the integer part and then we substract it from
                    // precision and use the result in the format! statement
                    let float_trunc_len = (float.trunc() as i32).to_string().len();
                    // Format without padding
                    if precision.is_some_and(|precision| precision > float_trunc_len) {
                        format!("{:.1$}", float, precision.unwrap() - float_trunc_len)
                    } else {
                        format!("{:.4}", float)
                    }
                };

                let formatted_e = {
                    let exponent = float.log10().floor();
                    let mantissa = float / 10f64.powf(exponent);
                    let sign = if float.is_sign_positive() { '+' } else { '-' };
                    // Precision in %g means max number of decimal digits in
                    // the mantissa. For that, we first calculate the length
                    // of the mantissa's int part and then we substract it from
                    // precision and use the result in the format! statement
                    let mantissa_trunc_len = (mantissa.trunc() as i32).to_string().len();
                    // Format without padding
                    if let Some(precision) = precision {
                        if precision > mantissa_trunc_len {
                            format!(
                                "{0:.1$}e{2}{3:02}",
                                mantissa,
                                precision - mantissa_trunc_len,
                                sign,
                                exponent
                            )
                        } else {
                            format!("{:.0}e{}{:02}", mantissa, sign, exponent)
                        }
                    } else {
                        format!("{}e{}{:02}", mantissa, sign, exponent)
                    }
                };

                // Use shortest formatted string
                let formatted_g = if formatted_f.len() < formatted_e.len() {
                    formatted_f
                } else {
                    formatted_e
                };

                // Pad to length
                let pad_width = pad_width as usize;
                let result = if pad_char == '0' {
                    format!("{:0>1$}", formatted_g, pad_width)
                } else {
                    format!("{:>1$}", formatted_g, pad_width)
                };
                res.extend_from_slice(result.as_bytes());
            }
            // TODO: more specifiers
            _ => unimplemented!(
                "Format character '{}'. Formatted up to index {}",
                specifier as char,
                format_char_idx
            ),
        }
    }

    log_dbg!("=> {:?}", std::str::from_utf8(&res));

    res
}

fn snprintf(
    env: &mut Environment,
    dest: MutPtr<u8>,
    n: GuestUSize,
    format: ConstPtr<u8>,
    args: DotDotDot,
) -> i32 {
    vsnprintf(env, dest, n, format, args.start())
}

fn vprintf(env: &mut Environment, format: ConstPtr<u8>, arg: VaList) -> i32 {
    log_dbg!(
        "vprintf({:?} ({:?}), ...)",
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), arg);
    // TODO: I/O error handling
    let _ = std::io::stdout().write_all(&res);
    res.len().try_into().unwrap()
}

fn vsnprintf(
    env: &mut Environment,
    dest: MutPtr<u8>,
    n: GuestUSize,
    format: ConstPtr<u8>,
    arg: VaList,
) -> i32 {
    log_dbg!(
        "vsnprintf({:?} {:?} {:?})",
        dest,
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), arg);
    let middle = if ((n - 1) as usize) < res.len() {
        &res[..(n - 1) as usize]
    } else {
        &res[..]
    };

    let dest_slice = env.mem.bytes_at_mut(dest, n);
    for (i, &byte) in middle.iter().chain(b"\0".iter()).enumerate() {
        dest_slice[i] = byte;
    }

    res.len().try_into().unwrap()
}

fn vsprintf(env: &mut Environment, dest: MutPtr<u8>, format: ConstPtr<u8>, arg: VaList) -> i32 {
    log_dbg!(
        "vsprintf({:?}, {:?} ({:?}), ...)",
        dest,
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), arg);

    let dest_slice = env
        .mem
        .bytes_at_mut(dest, (res.len() + 1).try_into().unwrap());
    for (i, &byte) in res.iter().chain(b"\0".iter()).enumerate() {
        dest_slice[i] = byte;
    }

    res.len().try_into().unwrap()
}

fn sprintf(env: &mut Environment, dest: MutPtr<u8>, format: ConstPtr<u8>, args: DotDotDot) -> i32 {
    log_dbg!(
        "sprintf({:?}, {:?} ({:?}), ...)",
        dest,
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), args.start());

    let dest_slice = env
        .mem
        .bytes_at_mut(dest, (res.len() + 1).try_into().unwrap());
    for (i, &byte) in res.iter().chain(b"\0".iter()).enumerate() {
        dest_slice[i] = byte;
    }

    res.len().try_into().unwrap()
}

fn printf(env: &mut Environment, format: ConstPtr<u8>, args: DotDotDot) -> i32 {
    log_dbg!(
        "printf({:?} ({:?}), ...)",
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), args.start());
    // TODO: I/O error handling
    let _ = std::io::stdout().write_all(&res);
    res.len().try_into().unwrap()
}

// TODO: more printf variants

fn sscanf(env: &mut Environment, src: ConstPtr<u8>, format: ConstPtr<u8>, args: DotDotDot) -> i32 {
    log_dbg!(
        "sscanf({:?}, {:?} ({:?}), ...)",
        src,
        format,
        env.mem.cstr_at_utf8(format)
    );

    let mut args = args.start();

    let mut src_ptr = src.cast_mut();
    let mut format_char_idx = 0;

    let mut matched_args = 0;

    loop {
        let c = env.mem.read(format + format_char_idx);
        format_char_idx += 1;

        if c == b'\0' {
            break;
        }
        if c != b'%' {
            let cc = env.mem.read(src_ptr);
            if c != cc {
                return matched_args - 1;
            }
            src_ptr += 1;
            continue;
        }

        let specifier = env.mem.read(format + format_char_idx);
        format_char_idx += 1;

        match specifier {
            b'd' => {
                let mut val: i32 = 0;
                while let c @ b'0'..=b'9' = env.mem.read(src_ptr) {
                    val = val * 10 + (c - b'0') as i32;
                    src_ptr += 1;
                }
                let c_int_ptr: ConstPtr<i32> = args.next(env);
                env.mem.write(c_int_ptr.cast_mut(), val);
            }
            // TODO: more specifiers
            _ => unimplemented!("Format character '{}'", specifier as char),
        }

        matched_args += 1;
    }

    matched_args
}

fn fprintf(
    env: &mut Environment,
    stream: MutPtr<FILE>,
    format: ConstPtr<u8>,
    args: DotDotDot,
) -> i32 {
    log_dbg!(
        "fprintf({:?}, {:?} ({:?}), ...)",
        stream,
        format,
        env.mem.cstr_at_utf8(format)
    );

    let res = printf_inner::<false, _>(env, |mem, idx| mem.read(format + idx), args.start());
    // TODO: I/O error handling
    match env.mem.read(stream).fd {
        STDOUT_FILENO => _ = std::io::stdout().write_all(&res),
        STDERR_FILENO => _ = std::io::stderr().write_all(&res),
        _ => unimplemented!(),
    }
    res.len().try_into().unwrap()
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(sscanf(_, _, _)),
    export_c_func!(snprintf(_, _, _, _)),
    export_c_func!(vprintf(_, _)),
    export_c_func!(vsnprintf(_, _, _, _)),
    export_c_func!(vsprintf(_, _, _)),
    export_c_func!(sprintf(_, _, _)),
    export_c_func!(printf(_, _)),
    export_c_func!(fprintf(_, _, _)),
];