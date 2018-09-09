use std::any::Any;
use std::rc::Rc;

use ast::{NativeFunction, Value};
use interpreter::{Scope, ScopeChain};

/// Native "print" function
pub struct NFPrint;

/// Native "println" function
pub struct NFPrintLn;

impl NativeFunction for NFPrint {
    /// Execute the "print" NativeFunction
    ///
    /// Prints all arguments in turn to stdout.
    fn execute<'src>(&self, _scopes: &mut ScopeChain<'src>, args: &Vec<Value<'src>>) -> Value<'src> {
        for arg in args {
            match arg {
                Value::Int(x)  => print!("{}", x),
                Value::Real(x) => print!("{}", x),
                Value::Str(x)  => print!("{}", x),
                _ => print!("{:?}", arg),
            };
        }
        Value::None
    }
    fn as_any(&self) -> &Any {
        self
    }
}

impl NativeFunction for NFPrintLn {
    /// Execute the "println" NativeFunction
    ///
    /// Prints all arguments in turn to stdout, followed by a newline.
    fn execute<'src>(&self, _scopes: &mut ScopeChain<'src>, args: &Vec<Value<'src>>) -> Value<'src> {
        for arg in args {
            match arg {
                Value::Int(x)  => print!("{}", x),
                Value::Real(x) => print!("{}", x),
                Value::Str(x)  => print!("{}", x),
                _ => println!("{:?}", arg),
            };
        }
        println!("");
        Value::None
    }

    fn as_any(&self) -> &Any {
        self
    }
}

/// Takes a Scope and inserts the NativeFunctions defined in this runtime module for use within
/// scripts.
pub fn insert_native_functions(scope: &mut Scope) {
    scope
        .native_funcs
        .insert("print", Rc::new(NFPrint {}));
    scope
        .native_funcs
        .insert("println", Rc::new(NFPrintLn {}));
}
