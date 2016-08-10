extern crate getopts;
extern crate notify;

extern crate rust_experiments;
use rust_experiments::cli::tail::tail;

use getopts::Options;
use notify::{RecommendedWatcher, Watcher};
use std::env;
use std::fs::File;
use std::io::{Write, stdin, stdout};
use std::sync::mpsc::channel;

const LINES_OPTION: &'static str = "n";
const FOLLOW_OPTION: &'static str = "f";
const HELP_OPTION: &'static str = "h";
const DEFAULT_LINES_NUMBER: usize = 10;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn tail_looped(input: &mut File,
               input_file_name: &str,
               output: &mut Write,
               limit: usize)
               -> notify::Result<()> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = try!(Watcher::new(tx));
    try!(watcher.watch(input_file_name));

    loop {
        tail(input, output, limit);

        match rx.recv() {
            Ok(_) => {}
            Err(message) => println!("Error: {}", message),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut opts = Options::new();

    opts.optopt(LINES_OPTION, "lines", "output last K lines", "K");
    opts.optflag(FOLLOW_OPTION,
                 "follow",
                 "output appended data as the file grows");
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
        tail(&mut input, &mut stdout(), limit);
    } else {
        let file_name = &matches.free[0];
        match File::open(file_name) {
            Ok(mut input) => {
                if matches.opt_present(FOLLOW_OPTION) {
                    let _ = tail_looped(&mut input, file_name, &mut stdout(), limit);
                } else {
                    tail(&mut input, &mut stdout(), limit)
                }
            }
            Err(text) => println!("Error: {}", text),
        }
    }
}
