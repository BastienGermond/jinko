//! This module contains all builtin functions declared in the jinko interpreter

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::ffi;
use crate::instance::{FromObjectInstance, ToObjectInstance};
use crate::{Context, Instruction, JkBool, JkInt, JkString, ObjectInstance};

type Args = Vec<Box<dyn Instruction>>;
type BuiltinFn = fn(&mut Context, Args) -> Option<ObjectInstance>;

/// Contains the various components declared during the interpreter's initialization
pub struct Builtins {
    functions: HashMap<String, BuiltinFn>,
}

/// Get the length of a string. Defined in stdlib/string.jk
/// The first argument is the string to get the length of
fn string_len(ctx: &mut Context, args: Args) -> Option<ObjectInstance> {
    let arg0 = args[0].execute(ctx).unwrap();
    let jk_string = JkString::from_instance(&arg0);

    Some(JkInt::from(jk_string.0.len() as i64).to_instance())
}

/// Concatenate two strings together. Defined in stdlib/string.jk
fn string_concat(ctx: &mut Context, args: Args) -> Option<ObjectInstance> {
    let lhs = JkString::from_instance(&args[0].execute(ctx).unwrap()).0;
    let rhs = JkString::from_instance(&args[1].execute(ctx).unwrap()).0;

    Some(JkString::from(format!("{}{}", lhs, rhs)).to_instance())
}

fn string_display(ctx: &mut Context, args: Args) -> Option<ObjectInstance> {
    let s = JkString::from_instance(&args[0].execute(ctx).unwrap()).0;

    io::stdout().lock().write_all(s.as_bytes()).unwrap();

    None
}

fn string_display_err(ctx: &mut Context, args: Args) -> Option<ObjectInstance> {
    let s = JkString::from_instance(&args[0].execute(ctx).unwrap()).0;

    io::stderr().lock().write_all(s.as_bytes()).unwrap();

    None
}

fn string_is_empty(ctx: &mut Context, args: Args) -> Option<ObjectInstance> {
    let s = JkString::from_instance(&args[0].execute(ctx).unwrap()).0;

    Some(JkBool::from(s.is_empty()).to_instance())
}

fn string_equals(ctx: &mut Context, args: Args) -> Option<ObjectInstance> {
    let lhs = JkString::from_instance(&args[0].execute(ctx).unwrap()).0;
    let rhs = JkString::from_instance(&args[1].execute(ctx).unwrap()).0;

    Some(JkBool::from(lhs == rhs).to_instance())
}

/// Link with a given library at runtime
fn ffi_link_with(ctx: &mut Context, args: Args) -> Option<ObjectInstance> {
    let lib_path = JkString::from_instance(&args[0].execute(ctx).unwrap()).0;

    if let Err(e) = ffi::link_with(ctx, PathBuf::from(&lib_path)) {
        ctx.error(e.with_msg(format!("couldn't link with library `{}`", &lib_path)));
    }

    None
}

// Get an argument from the argument vector at a certain index
fn arg_get(ctx: &mut Context, args: Args) -> Option<ObjectInstance> {
    let idx = JkInt::from_instance(&args[0].execute(ctx).unwrap()).0;

    // FIXME: Do not fetch arguments continuously
    let mut args = std::env::args();

    // FIXME: Is this cast valid?
    let result_string = args.nth(idx as usize).unwrap_or(String::new());

    Some(JkString::from(result_string).to_instance())
}

impl Builtins {
    fn add(&mut self, name: &'static str, builtin_fn: BuiltinFn) {
        self.functions.insert(String::from(name), builtin_fn);
    }

    /// Create a new instance of builtins, with pre-defined functions
    pub fn new() -> Builtins {
        let mut builtins = Builtins {
            functions: HashMap::new(),
        };

        builtins.add("__builtin_string_len", string_len);
        builtins.add("__builtin_string_concat", string_concat);
        builtins.add("__builtin_string_display", string_display);
        builtins.add("__builtin_string_display_err", string_display_err);
        builtins.add("__builtin_string_is_empty", string_is_empty);
        builtins.add("__builtin_string_equals", string_equals);
        builtins.add("__builtin_ffi_link_with", ffi_link_with);
        builtins.add("__builtin_arg_get", arg_get);

        builtins
    }

    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn get(&self, builtin: &str) -> Option<&BuiltinFn> {
        self.functions.get(builtin)
    }
}

impl Default for Builtins {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::jinko;

    #[test]
    fn t_string_builtins_are_valid() {
        jinko! {
            __builtin_string_len("jk");
            __builtin_string_concat("file", ".jk");
            __builtin_string_display("to display");
            __builtin_string_display_err("to display on err");
        };
    }

    #[test]
    fn t_ffi_builtins_are_valid() {
        jinko! {
            __builtin_ffi_link_with("tests/fixtures/clib/lib.so");
        };
    }
}
