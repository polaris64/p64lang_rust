extern crate p64lang;

use std::io::{self, Read};

use p64lang::ast::Scope;
use p64lang::interpret;
use p64lang::runtime::DefaultScriptInterface;

fn main() {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Unable to read input");

    let scope = Scope::new();
    let res = interpret(&buffer, scope, &mut DefaultScriptInterface::new());
    println!("Result: {:?}", res);
}
