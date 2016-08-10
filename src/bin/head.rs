extern crate getopts;

extern crate rust_experiments;
use rust_experiments::cli::head::head;

use getopts::Options;
use std::env;
use std::fs::File;
use std::io::{stdin, stdout};

const LINES_OPTION: &'static str = "n";
const HELP_OPTION: &'static str = "h";
const DEFAULT_LINES_NUMBER: usize = 10;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut opts = Options::new();

    opts.optopt(LINES_OPTION, "lines", "output first K lines", "K");
    opts.optflag(HELP_OPTION, "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(message) => {
            println!("Error: {}", message);
            print_usage(program, opts);
            return;
        }
    };

    if matches.opt_present(HELP_OPTION) {
        print_usage(program, opts);
        return;
    }

    let mut limit = DEFAULT_LINES_NUMBER;
    if matches.opt_present(LINES_OPTION) {
        if let Some(text) = matches.opt_str(LINES_OPTION) {
            match text.parse::<usize>() {
                Ok(number) => {
                    limit = number;
                }
                Err(message) => {
                    println!("Error: {}", message);
                    print_usage(program, opts);
                    return;
                }
            }
        } else {
            print_usage(program, opts);
            return;
        }
    }

    if matches.free.is_empty() {
        let mut input = stdin();
        head(&mut input, &mut stdout(), limit);
    } else {
        let file_name = &matches.free[0];
        match File::open(file_name) {
            Ok(mut input) => head(&mut input, &mut stdout(), limit),
            Err(text) => println!("Error: {}", text),
        }
    }
}
