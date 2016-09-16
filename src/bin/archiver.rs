extern crate rust_experiments;
extern crate walkdir;

#[macro_use]
extern crate clap;

use clap::{ArgGroup, ArgMatches, App};
use rust_experiments::encoding::bitreader::BitReader;
use rust_experiments::encoding::bitwriter::BitWriter;
use rust_experiments::encoding::huffman::{HuffmanEncoder, HuffmanDecoder};
use rust_experiments::format::size_to_human_readable;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};
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

type Filenames = Vec<String>;
type FileEntries = Vec<FileEntry>;
type Encoder<'a> = HuffmanEncoder<&'a mut File>;

pub fn create_archive(output_filename: &str, files: Filenames) -> Result<()> {
    let mut entries = files_to_entries(files);

    try!(create_parent_directories(output_filename));
    let (header_length_bits, mut writer) = try!(write_header(output_filename, &entries));

    let mut encoder = HuffmanEncoder::new(writer.get_mut());
    try!(write_compressed_data(&mut entries, &mut encoder, header_length_bits));

    try!(write_offsets(&entries, &mut encoder));
    try!(encoder.get_output_mut().flush());

    Ok(())
}

pub fn extract_archive(input_filename: &str, files: Filenames) -> Result<()> {
    let match_all_files = files.is_empty();

    let (entries, mut reader) = try!(load_header(input_filename));
    let mut decoder = try!(HuffmanDecoder::new(reader.get_mut()));

    let mut unpacked = 0;

    for ref entry in &entries {
        // FIXME: poor performance
        let file_matched = match_all_files ||
                           files.iter()
            .any(|i| filename_matches(i, &entry.filename));
        if file_matched {
            print!("unpacking {} ...", entry.filename);
            try!(create_parent_directories(entry.filename.as_str()));
            let mut output = try!(File::create(entry.filename.clone()));
            try!(decoder.get_reader_mut().set_position(entry.offset_bits));
            try!(decoder.decode(&mut output, entry.offset_bits, entry.size_bytes * 8));
            try!(output.flush());
            println!(" ok");
            unpacked += 1;
        }
    }

    if unpacked == 0 {
        let e = Error::new(ErrorKind::NotFound, "nothing to unpack");
        Err(e)
    } else {
        Ok(())
    }
}

pub fn list_archive(input_filename: &str, files: Filenames) -> Result<()> {
    let (entries, _) = try!(load_header(input_filename));
    let match_all_files = files.is_empty();

    for ref entry in &entries {
        // FIXME: poor performance
        let file_matched = match_all_files ||
                           files.iter()
            .any(|i| filename_matches(i, &entry.filename));

        if file_matched {
            let size = size_to_human_readable(entry.size_bytes as f64);
            println!("{:15}{}", size, entry.filename);
        }
    }
    Ok(())
}

fn do_checked_main(matches: ArgMatches, files: Filenames) -> Result<()> {
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
        .args_from_usage("[FILE]... 'Filenames to compress or extract'
                            -C <current_directory> 'Change current directory'
                            -c <archive.huff> 'Create archive'
                            -l <archive.huff> 'List contents'
                            -x <archive.huff> 'Extract archive'")
        .group(ArgGroup::with_name("cxl")
            .args(&["c", "x", "l"])
            .required(true))
        .get_matches();

    let files = values_t!(matches, "FILE", String).unwrap_or_else(|_| vec![]);

    match do_checked_main(matches, files) {
        Ok(_) => println!("OK"),
        Err(e) => println!("Error: {:?}", e),
    }
}

fn files_to_entries(files: Filenames) -> FileEntries {
    let mut entries = vec![];

    for i in &files {
        let filenames = WalkDir::new(i)
            .into_iter()
            .filter_map(|f| f.ok())
            .filter(|f| f.file_type().is_file() && f.path().to_str().is_some())
            .map(|f| f.path().to_str().unwrap().to_string());

        let mut new_entries: FileEntries = filenames.filter_map(|f| {
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

fn write_header(output_filename: &str, entries: &FileEntries) -> Result<(u64, BitWriter<File>)> {
    let entries_length = entries.len() as u64;
    let output = try!(File::create(output_filename));
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

    let header_length_bits = writer.position();
    let result = (header_length_bits, writer);
    Ok(result)
}

fn write_compressed_data(entries: &mut FileEntries,
                         encoder: &mut Encoder,
                         header_length_bits: u64)
                         -> Result<()> {
    for entry in entries.iter() {
        let f = try!(File::open(entry.filename.clone()));
        try!(encoder.analyze(f));
    }
    try!(encoder.analyze_finish());

    for entry in entries.iter_mut() {
        let f = try!(File::open(entry.filename.clone()));
        entry.offset_bits = encoder.position() + header_length_bits;
        try!(encoder.compress(f));
    }

    encoder.compress_finish()
}

fn write_offsets(entries: &FileEntries, encoder: &mut Encoder) -> Result<()> {
    let entries_length = entries.len() as u64;
    let skip_length = mem::size_of_val(&entries_length);
    try!(encoder.get_output_mut().seek(SeekFrom::Start(skip_length as u64)));

    for entry in entries {
        try!(encoder.get_writer_mut().write_u64(entry.offset_bits));

        let skip_length = mem::size_of_val(&entry.size_bytes) +
                          mem::size_of_val(&entry.filename_length_bytes) +
                          entry.filename_length_bytes as usize;
        let skip_length = skip_length as i64;
        try!(encoder.get_output_mut().seek(SeekFrom::Current(skip_length)));
    }

    Ok(())
}

fn load_header(input_filename: &str) -> Result<(FileEntries, BitReader<File>)> {
    let input = try!(File::open(input_filename));
    let mut reader = BitReader::new(input);
    let entries_length = try!(reader.read_u64());
    let mut entries = Vec::with_capacity(entries_length as usize);

    for _ in 0..entries_length {
        let offset_bits = try!(reader.read_u64());
        let size_bytes = try!(reader.read_u64());
        let filename_length_bytes = try!(reader.read_u64());

        let mut filename = Vec::with_capacity(filename_length_bytes as usize);
        for _ in 0..filename_length_bytes {
            let ch = try!(reader.read_u8());
            filename.push(ch);
        }
        let filename = String::from_utf8_lossy(&filename[..]).into_owned();

        let entry = FileEntry {
            offset_bits: offset_bits,
            size_bytes: size_bytes,
            filename_length_bytes: filename_length_bytes,
            filename: filename,
        };
        entries.push(entry);
    }

    Ok((entries, reader))
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

fn filename_matches(pattern: &String, filename: &String) -> bool {
    let pattern_is_directory = pattern.ends_with("/");
    (pattern_is_directory && filename.starts_with(pattern.as_str())) || (pattern == filename)
}
