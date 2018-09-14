#![no_std]

#![feature(alloc)]
#[macro_use]
extern crate alloc;

extern crate p64lang;
extern crate wasm_bindgen;

use core::any::Any;
use alloc::fmt::Write;
use alloc::rc::Rc;
use alloc::string::String;

use wasm_bindgen::prelude::*;

use p64lang::ast::{NativeFunction, Value};
use p64lang::interpreter::{Scope, ScopeChain};
use p64lang::interpret;

struct NFPrint;
impl NativeFunction for NFPrint {
    fn execute<'src>(&self, _scopes: &mut ScopeChain<'src>, args: &[Value<'src>]) -> Value<'src> {
        let mut buf = String::new();
        for arg in args {
            match arg {
                Value::Int(x)  => write!(buf, "{}", x).unwrap_or_default(),
                Value::Real(x) => write!(buf, "{}", x).unwrap_or_default(),
                Value::Str(x)  => write!(buf, "{}", x).unwrap_or_default(),
                _ => write!(buf, "{:?}", arg).unwrap_or_default(),
            };
        }
        js_print(buf.as_str(), false);
        Value::None
    }

    fn as_any(&self) -> &Any {
        self
    }
}

struct NFPrintLn;
impl NativeFunction for NFPrintLn {
    fn execute<'src>(&self, _scopes: &mut ScopeChain<'src>, args: &[Value<'src>]) -> Value<'src> {
        let mut buf = String::new();
        for arg in args {
            match arg {
                Value::Int(x)  => write!(buf, "{}", x).unwrap_or_default(),
                Value::Real(x) => write!(buf, "{}", x).unwrap_or_default(),
                Value::Str(x)  => write!(buf, "{}", x).unwrap_or_default(),
                _ => write!(buf, "{:?}", arg).unwrap_or_default(),
            };
        }
        js_print(buf.as_str(), true);
        Value::None
    }

    fn as_any(&self) -> &Any {
        self
    }
}

#[wasm_bindgen(module = "./index.js")]
extern {
    fn js_print(s: &str, nl: bool);
}

#[wasm_bindgen]
pub fn interpret_str(src: &str) -> String {
    let mut scope = Scope::new();
    scope.native_funcs.insert("print",   Rc::new(NFPrint   {}));
    scope.native_funcs.insert("println", Rc::new(NFPrintLn {}));
    let res = interpret(src, scope);
    format!("Result: {:?}", res.exec_result)
}
