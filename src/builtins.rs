//! This module contains all builtin functions declared in the jinko interpreter

use std::collections::HashMap;
use std::io::{self, Write};

use crate::instance::{FromObjectInstance, ToObjectInstance};
use crate::{Context, Instruction, JkInt, JkString, ObjectInstance};

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

        builtins
    }

    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn get(&self, builtin: &str) -> Option<&BuiltinFn> {
        self.functions.get(builtin)
    }
}
