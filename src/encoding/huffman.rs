use encoding::bitreader::BitReader;
use encoding::bitwriter::BitWriter;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Error, ErrorKind, Read, Result, Seek, Write};
use std::mem;
use structs::binary_tree::BinaryTree;
use structs::bitset::BitSet;

type Char = Vec<u8>;
type CharSlice<'a> = &'a [u8];
type DictLength = u16;

type CodeLength = u8;
type CodeData = u16;

#[derive(PartialEq, Debug)]
enum State {
    Initial,
    Analyzed,
    Compressed,
}

#[derive(Clone, PartialEq, Debug)]
struct NodeData {
    chars: HashSet<Char>,
    weight: u64,
}

type Tree = BinaryTree<NodeData>;

#[derive(PartialEq, Eq, Hash, Debug)]
struct Code {
    length: CodeLength,
    data: CodeData,
}

fn read_char<R>(input: &mut BitReader<R>, max_char_length: usize) -> Option<Char>
    where R: Read
{
    let mut result = Vec::with_capacity(max_char_length);
    for _ in 0..max_char_length {
        match input.read_u8() {
            Ok(buffer) => result.push(buffer),
            Err(_) => break,
        }
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

include!("huffman_encoder.rs");
include!("huffman_decoder.rs");
include!("huffman_tests.rs");
