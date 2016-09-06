use encoding::bitreader::BitReader;
use encoding::bitwriter::BitWriter;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Read, Result, Seek, SeekFrom, Write};
use structs::binary_tree::BinaryTree;
use structs::bitset::BitSet;

include!("huffman_encoder.rs");
include!("huffman_decoder.rs");
include!("huffman_tests.rs");
