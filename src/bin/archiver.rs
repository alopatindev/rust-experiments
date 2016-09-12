#![feature(test)]

extern crate rust_experiments;
extern crate walkdir;

#[macro_use]
extern crate clap;

use clap::{Arg, ArgGroup, ArgMatches, App};
use rust_experiments::encoding::bitreader::BitReader;
use rust_experiments::encoding::bitwriter::BitWriter;
use rust_experiments::encoding::huffman::{HuffmanEncoder, HuffmanDecoder};
use std::env;
use std::fs;
use std::io::{Error, ErrorKind, Result, Read, Seek, SeekFrom, Write};
use std::mem;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct FileEntry {
    offset_bits: u64,
    size_bytes: u64,
    filename_length_bytes: u64,
    filename: String,
}

pub fn create_archive(output_filename: &str, files: Vec<String>) -> Result<()> {
    let mut entries = files_to_entries(files);
    let entries_length = entries.len() as u64;

    try!(create_parent_directories(output_filename));
    let mut writer = try!(write_header(output_filename, &entries));

    let mut encoder = HuffmanEncoder::new(writer.get_mut());

    for entry in entries.iter() {
        let f = try!(fs::File::open(entry.filename.clone()));
        try!(encoder.analyze(f));
    }
    try!(encoder.analyze_finish());

    for entry in entries.iter_mut() {
        let f = try!(fs::File::open(entry.filename.clone()));
        try!(encoder.compress(f));
        entry.offset_bits = encoder.position();
    }
    encoder.compress_finish();

    let skip_length = mem::size_of_val(&entries_length);
    try!(encoder.get_output_mut().seek(SeekFrom::Start(skip_length as u64)));

    for entry in entries {
        try!(encoder.get_writer_mut().write_u64(entry.offset_bits));

        let skip_length = mem::size_of_val(&entry.offset_bits) +
                          mem::size_of_val(&entry.size_bytes) +
                          mem::size_of_val(&entry.filename_length_bytes) +
                          entry.filename_length_bytes as usize;
        try!(encoder.get_output_mut().seek(SeekFrom::Current(skip_length as i64)));
    }

    try!(encoder.get_output_mut().flush());

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
        Err(e) => println!("Error: {:?}", e),
    }
}

fn files_to_entries(files: Vec<String>) -> Vec<FileEntry> {
    let mut entries = vec![];

    for i in &files {
        let filenames = WalkDir::new(i)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().is_file() && f.path().to_str().is_some())
            .map(|f| f.path().to_str().unwrap().to_string());

        let mut new_entries: Vec<FileEntry> = filenames.filter_map(|f| {
                match fs::metadata(f.clone()) {
                    Ok(meta) => {
                        let size = meta.len();
                        let entry = FileEntry {
                            offset_bits: 0,
                            size_bytes: size,
                            filename_length_bytes: f.len() as u64,
                            filename: f,
                        };
                        Some(entry)
                    }
                    Err(_) => None,
                }
            })
            .collect();
        entries.append(&mut new_entries);
    }

    entries
}

fn write_header(output_filename: &str, entries: &Vec<FileEntry>) -> Result<BitWriter<fs::File>> {
    let entries_length = entries.len() as u64;
    let output = try!(fs::File::create(output_filename));
    let mut writer = BitWriter::new(output);

    try!(writer.write_u64(entries_length));
    for ref entry in entries {
        try!(writer.write_u64(entry.offset_bits));
        try!(writer.write_u64(entry.size_bytes));
        try!(writer.write_u64(entry.filename_length_bytes));
        for &ch in entry.filename.as_bytes() {
            try!(writer.write_u8(ch));
        }
    }

    Ok(writer)
}

fn create_parent_directories(filename: &str) -> Result<()> {
    let path = Path::new(filename);

    let e = Error::new(ErrorKind::InvalidInput,
                       format!("'{}' has no directory", filename));

    if let Some(directory) = path.parent() {
        let result = fs::create_dir_all(directory);
        return if result.is_ok() {
            assert!(directory.metadata().unwrap().is_dir());
            Ok(())
        } else {
            Err(e)
        };
    } else {
        Err(e)
    }
}
