extern crate lalrpop;

fn main() {
    println!("Building parser...");
    lalrpop::process_root().unwrap()
}
