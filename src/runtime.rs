use std::any::Any;
use std::rc::Rc;

use ast::{NativeFunction, Scope, ScopeChain, Value};

pub struct NFPrint;
pub struct NFPrintLn;

impl NativeFunction for NFPrint {
    fn execute(&self, _scopes: &mut ScopeChain, args: &Vec<Value>) -> Value {
        for arg in args {
            match arg {
                Value::Int(x) => print!("{}", x),
                Value::Real(x) => print!("{}", x),
                Value::Str(x) => print!("{}", x),
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
    fn execute(&self, _scopes: &mut ScopeChain, args: &Vec<Value>) -> Value {
        for arg in args {
            match arg {
                Value::Int(x) => println!("{}", x),
                Value::Real(x) => println!("{}", x),
                Value::Str(x) => println!("{}", x),
                _ => println!("{:?}", arg),
            };
        }
        Value::None
    }
    fn as_any(&self) -> &Any {
        self
    }
}

pub fn insert_native_functions(scope: &mut Scope) {
    scope
        .native_funcs
        .insert("print".to_string(), Rc::new(NFPrint {}));
    scope
        .native_funcs
        .insert("println".to_string(), Rc::new(NFPrintLn {}));
}
