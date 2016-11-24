extern crate rust_experiments;

use rust_experiments::interpreters::brainfuck::VM;
use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Read, Result};

fn do_checked_main(filename: String) -> Result<()> {
    let mut file = try!(File::open(filename));
    let mut program = String::new();
    try!(file.read_to_string(&mut program));

    let mut vm = VM::new(program, stdin(), stdout());
    vm.run();

    Ok(())
}

fn print_usage(executable: String) {
    println!("Brainfuck Interpreter");
    println!("Usage: {} program.bf", executable);
}

pub fn main() {
    let mut args = env::args();
    let executable = args.next().unwrap();
    match args.next() {
        Some(filename) => {
            match do_checked_main(filename) {
                Ok(_) => println!("OK"),
                Err(e) => println!("Error: {:?}", e),
            }
        }
        None => print_usage(executable),
    }
}
