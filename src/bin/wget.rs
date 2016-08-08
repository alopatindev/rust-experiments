extern crate getopts;

extern crate rust_experiments;
use rust_experiments::cli::wget::download;

use getopts::Options;
use std::env;

const HELP_OPTION: &'static str = "h";
const CONTINUE_OPTION: &'static str = "c";
const OUTPUT_DOCUMENT_OPTION: &'static str = "O";

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] URL", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];
    let mut opts = Options::new();

    opts.optflag(CONTINUE_OPTION,
                 "continue",
                 "resume getting a partially-downloaded file");
    opts.optopt(OUTPUT_DOCUMENT_OPTION,
                "output-document",
                "write documents to FILE",
                "FILE");
    opts.optflag(HELP_OPTION, "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(message) => {
            println!("Error: {}", message);
            print_usage(program, opts);
            return;
        }
    };

    if matches.opt_present(HELP_OPTION) || matches.free.is_empty() {
        print_usage(program, opts);
        return;
    }

    let url = &matches.free[0];

    let continue_partial = matches.opt_present(CONTINUE_OPTION);

    let mut output_document: Option<String> = None;
    if matches.opt_present(OUTPUT_DOCUMENT_OPTION) {
        output_document = matches.opt_str(OUTPUT_DOCUMENT_OPTION);
        if output_document.is_none() {
            print_usage(program, opts);
            return;
        }
    }

    match download(url, output_document, continue_partial) {
        Ok(_) => println!("Success"),
        Err(text) => println!("Error: {:?}", text),
    }
}
