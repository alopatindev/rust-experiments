#![feature(test)]

extern crate rust_experiments;
extern crate walkdir;

#[macro_use]
extern crate clap;

use clap::{Arg, ArgGroup, ArgMatches, App};
use rust_experiments::encoding::huffman::{HuffmanEncoder, HuffmanDecoder};
use std::env;
use std::fs;
use std::io::{Result, Read, Write};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct FileEntry {
    offset_bits: u64,
    size_bytes: u64,
    filename_bytes: u64,
    filename: String,
}

fn create_archive(output_filename: &str, files: Vec<String>) -> Result<()> {
    for i in &files {
        let filenames = WalkDir::new(i)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().is_file() && f.path().to_str().is_some())
            .map(|f| f.path().to_str().unwrap().to_string());

        let mut entries: Vec<FileEntry> = filenames.map(|f| {
                FileEntry {
                    offset_bits: 0,
                    size_bytes: 0,
                    filename_bytes: f.len() as u64,
                    filename: f,
                }
            })
            .collect();

        // TODO: write entries number

        for entry in entries.iter_mut() {
            println!("{:?}", entry);
            // TODO: write initial entry
        }

        // TODO: write the data

        // TODO: seek to 0 + sizeof(entries number)

        for entry in entries {
            // TODO: update offset_bits
            // TODO: seek to the next entry
        }

        // TODO: flush
    }

    Ok(())
}

fn extract_archive(input_filename: &str, files: Vec<String>) -> Result<()> {
    unimplemented!();
}

fn list_archive(input_filename: &str, files: Vec<String>) -> Result<()> {
    unimplemented!();
}

fn do_checked_main(matches: ArgMatches, files: Vec<String>) -> Result<()> {
    if let Some(current_directory) = matches.value_of("C") {
        try!(env::set_current_dir(current_directory));
    }

    if let Some(output_filename) = matches.value_of("c") {
        create_archive(output_filename, files)
    } else if let Some(input_filename) = matches.value_of("x") {
        extract_archive(input_filename, files)
    } else if let Some(input_filename) = matches.value_of("l") {
        list_archive(input_filename, files)
    } else {
        println!("{}", matches.usage());
        Ok(())
    }
}

fn main() {
    let matches = App::new("Archiver")
        .args_from_usage("[FILE]... 'Files to compress or extract'
                            -C <current_directory> 'Change current directory'
                            -c <archive.huff> 'Create archive'
                            -l <archive.huff> 'List contents'
                            -x <archive.huff> 'Extract archive'")
        .group(ArgGroup::with_name("cxl")
            .args(&["c", "x", "l"])
            .required(true))
        .get_matches();

    let files: Vec<String> = values_t!(matches, "FILE", String)
        .unwrap_or_else(|_| vec![".".to_string()]);

    match do_checked_main(matches, files) {
        Ok(_) => println!("OK"),
        Err(_) => println!("Something went wrong"),
    }
}
