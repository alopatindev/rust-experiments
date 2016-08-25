extern crate getopts;

extern crate rust_experiments;

use getopts::{Matches, Options};
use rust_experiments::cli::wc;
use std::env;
use std::fs::File;
use std::io::{Read, stdin};

const BYTES_OPTION: &'static str = "c";
const CHARS_OPTION: &'static str = "m";
const LINES_OPTION: &'static str = "l";
const WORDS_OPTION: &'static str = "w";
const MAX_LINE_LENGTH_OPTION: &'static str = "L";
const HELP_OPTION: &'static str = "h";

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn do_work(input: &mut Read, matches: &Matches) {
    let counters = wc::count(input);

    if matches.opt_present(BYTES_OPTION) {
        println!("{}", counters.bytes);
    } else if matches.opt_present(CHARS_OPTION) {
        println!("{}", counters.characters);
    } else if matches.opt_present(LINES_OPTION) {
        println!("{}", counters.newlines);
    } else if matches.opt_present(WORDS_OPTION) {
        println!("{}", counters.words);
    } else if matches.opt_present(MAX_LINE_LENGTH_OPTION) {
        println!("{}", counters.max_line_length);
    } else {
        println!("\t{}\t{}\t{}",
                 counters.newlines,
                 counters.words,
                 counters.bytes);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut opts = Options::new();

    opts.optflag(BYTES_OPTION, "bytes", "print the byte counts");
    opts.optflag(CHARS_OPTION, "chars", "print the character counts");
    opts.optflag(LINES_OPTION, "lines", "print the newline counts");
    opts.optflag(WORDS_OPTION, "words", "print the word counts");
    opts.optflag(MAX_LINE_LENGTH_OPTION,
                 "max-line-length",
                 "print the length of the longest line");
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

    if matches.free.is_empty() {
        let mut input = stdin();
        do_work(&mut input, &matches);
    } else {
        let file_name = &matches.free[0];
        match File::open(file_name) {
            Ok(mut input) => do_work(&mut input, &matches),
            Err(text) => println!("Error: {}", text),
        }
    };
}
