extern crate p64lang;

use std::io::{self, Read};

use p64lang::ast::Scope;
use p64lang::interpret;
use p64lang::runtime::insert_native_functions;

fn main() {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("Unable to read input");

    let mut scope = Scope::new();
    insert_native_functions(&mut scope);
    let res = interpret(&buffer, scope);
    println!("Result: {:?}", res.exec_result);
}
