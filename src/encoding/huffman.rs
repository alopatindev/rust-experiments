use encoding::bitreader::BitReader;
use encoding::bitwriter::BitWriter;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Result, Seek, Write};
use structs::binary_tree::BinaryTree;

#[derive(Clone, PartialEq, Debug)]
pub struct NodeData {
    chars: HashSet<u8>,
    weight: u64,
}

pub type Tree = BinaryTree<NodeData>;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Code {
    length: u8,
    data: u8,
}

pub type CodesToChars = HashMap<Code, u8>;
pub type CharsToCodes = HashMap<u8, Code>;

pub fn compress<R, W>(input: &mut BitReader<R>, output: &mut BitWriter<W>) -> Result<u64>
    where R: Read + Seek,
          W: Write
{
    let mut input_bytes_read = 0;
    let tree = compression::build_tree(input, &mut input_bytes_read);
    let chars_to_codes = compression::build_dictionary(&tree);

    try!(compression::write_header(output, &chars_to_codes, &input_bytes_read));
    let result = compression::write_compressed(input, output, &chars_to_codes, &input_bytes_read);
    output.flush(); // FIXME
    result
}

pub fn decompress<R>(input: &mut BitReader<R>, output: &mut Write) -> Result<u64>
    where R: Read
{
    let codes_to_chars = try!(decompression::read_header(input));
    if codes_to_chars.is_empty() {
        try!(output.flush()); // FIXME
        return Ok(0);
    }

    let expected_uncompressed_bytes = try!(input.read_u64());

    let result =
        decompression::read_compressed(input, output, &codes_to_chars, expected_uncompressed_bytes);
    try!(output.flush()); // FIXME
    result
}

mod compression {
    use encoding::bitreader::BitReader;
    use encoding::bitwriter::BitWriter;
    use std::collections::{HashMap, HashSet, VecDeque};
    use std::io::{Read, Result, Seek, SeekFrom, Write};
    use structs::binary_tree::BinaryTree;
    use structs::bitset::BitSet;
    use super::*;

    pub fn write_header<W>(output: &mut BitWriter<W>,
                           chars_to_codes: &CharsToCodes,
                           input_bytes_read: &u64)
                           -> Result<()>
        where W: Write
    {
        if !chars_to_codes.is_empty() {
            let max_index = (chars_to_codes.len() - 1) as u8;
            try!(output.write_u8(max_index));
            for (&ch, code) in chars_to_codes {
                try!(output.write_u8(code.length));
                try!(output.write_u8(code.data));
                try!(output.write_u8(ch));
            }

            try!(output.write_u64(*input_bytes_read));
        }

        Ok(())
    }

    pub fn write_compressed<R, W>(input: &mut BitReader<R>,
                                  output: &mut BitWriter<W>,
                                  chars_to_codes: &CharsToCodes,
                                  input_bytes_read: &u64)
                                  -> Result<u64>
        where R: Read + Seek,
              W: Write
    {
        try!(input.seek(SeekFrom::Start(0)));

        let mut bytes_read = 0;
        let mut bits_written = 0;

        while bytes_read < *input_bytes_read {
            match input.read_u8() {
                Ok(buffer) => {
                    let code = chars_to_codes.get(&buffer).unwrap();
                    for i in 0..code.length {
                        let shifted_one = 1 << i;
                        let data = (code.data & shifted_one) > 0;
                        try!(output.write_bit(data));
                        bits_written += 1;
                    }
                    bytes_read += 1;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        output.flush();

        Ok(bits_written)
    }

    pub fn compute_leaves<R>(input: &mut BitReader<R>, input_bytes_read: &mut u64) -> Vec<Tree>
        where R: Read + Seek
    {
        let mut char_to_weight: HashMap<u8, u64> = HashMap::new();

        while let Ok(buffer) = input.read_u8() {
            char_to_weight.entry(buffer).or_insert(0);
            char_to_weight.get_mut(&buffer).map(|mut w| *w += 1);
            *input_bytes_read += 1;
        }

        let mut result = Vec::with_capacity(char_to_weight.len());
        for (&ch, &weight) in &char_to_weight {
            let chars = hashset!{ch};
            let data: NodeData = NodeData {
                chars: chars,
                weight: weight,
            };
            result.push(BinaryTree::new_leaf(data));
        }

        result
    }

    pub fn build_next_level(level: &VecDeque<Tree>, next_level: &mut VecDeque<Tree>) {
        let n = level.len();
        let mut k = n; // FIXME

        while k > 0 {
            let i = k - 1;
            let last_node_in_level = i == 0;
            let new_parent_has_same_weight = match next_level.front() {
                Some(tree) => tree.data().unwrap().weight <= level[i].data().unwrap().weight,
                None => false,
            };
            if last_node_in_level || new_parent_has_same_weight {
                let head = next_level.pop_front().unwrap();
                let parent = new_parent(&level[i], &head);
                next_level.push_front(parent);
                if last_node_in_level {
                    break;
                }
                k -= 1;
            } else {
                let parent = new_parent(&level[i], &level[i - 1]);
                next_level.push_front(parent);
                k -= 2;
            }
        }
    }

    pub fn new_parent(left: &Tree, right: &Tree) -> Tree {
        let left_chars = &left.data().unwrap().chars;
        let right_chars = &right.data().unwrap().chars;

        let chars = left_chars.union(right_chars).cloned().collect::<HashSet<u8>>();
        let weight = left.data().unwrap().weight + right.data().unwrap().weight;

        let data = NodeData {
            chars: chars,
            weight: weight,
        };
        Tree::new(data, left, right)
    }

    pub fn build_tree<R>(chars: &mut BitReader<R>, input_bytes_read: &mut u64) -> Tree
        where R: Read + Seek
    {
        let mut leaves = compute_leaves(chars, input_bytes_read);
        leaves.sort_by_key(|tree| tree.data().unwrap().weight);
        leaves.reverse();

        let mut level = VecDeque::with_capacity(leaves.len());
        for i in &leaves {
            level.push_back(i.clone());
        }

        if level.is_empty() {
            return BinaryTree::new_empty();
        }

        let new_level = |level: &VecDeque<Tree>| -> VecDeque<Tree> {
            let length = level.len() / 2 + 1;
            VecDeque::with_capacity(length)
        };

        let mut next_level = new_level(&level);

        loop {
            let found_root = next_level.is_empty() && level.len() == 1;
            if found_root {
                break;
            } else {
                build_next_level(&level, &mut next_level);
                level = next_level;
                next_level = new_level(&level);
            }
        }

        level[0].clone()
    }

    pub fn compute_code(ch: u8, tree: &Tree) -> Code {
        let mut tree = tree.clone();

        let mut code = BitSet::new();
        let mut length = 0;

        loop {
            if tree.left_data().is_some() && tree.left_data().unwrap().chars.contains(&ch) {
                tree = tree.left();
            } else if tree.right_data().is_some() &&
                      tree.right_data().unwrap().chars.contains(&ch) {
                code.insert(length);
                tree = tree.right();
            } else {
                break;
            }
            length += 1;
        }

        assert!(tree.is_leaf());

        Code {
            length: length as u8,
            data: code.as_slice()[0] as u8,
        }
    }

    pub fn build_dictionary(tree: &Tree) -> CharsToCodes {
        let mut result = HashMap::new();

        if let Some(data) = tree.data() {
            for &ch in &data.chars {
                let code = compute_code(ch, tree);
                result.insert(ch, code);
            }
        }

        assert!(result.len() <= 256);

        result
    }
}

mod decompression {
    use encoding::bitreader::BitReader;
    use std::io::{Read, Result, Write};
    use super::*;

    pub fn read_header<R>(input: &mut BitReader<R>) -> Result<CodesToChars>
        where R: Read
    {
        let len = match input.read_u8() {
            Ok(max_index) => (max_index as usize) + 1,
            Err(_) => 0,
        };

        let mut result = CodesToChars::with_capacity(len);

        for _ in 0..len {
            let code_length = try!(input.read_u8());
            let code_data = try!(input.read_u8());
            let ch = try!(input.read_u8());
            let code = Code {
                length: code_length,
                data: code_data,
            };
            result.insert(code, ch);
        }

        Ok(result)
    }

    pub fn read_compressed<R>(input: &mut BitReader<R>,
                              output: &mut Write,
                              codes_to_chars: &CodesToChars,
                              uncompressed_bytes: u64)
                              -> Result<u64>
        where R: Read
    {
        assert!(uncompressed_bytes > 0);

        let mut read_bytes = 0;
        if uncompressed_bytes == 1 {
            // FIXME: remove
            assert_eq!(1, codes_to_chars.len());
            let ch = *codes_to_chars.values().next().unwrap();
            try!(output.write_all(&[ch]));
            read_bytes += 1;
        } else {
            while read_bytes < uncompressed_bytes {
                match read_char(input, codes_to_chars) {
                    Some(ch) => {
                        try!(output.write_all(&[ch]));
                        read_bytes += 1;
                    }
                    None => unreachable!(),
                }
            }
        }

        try!(output.flush());

        let read_bits = read_bytes * 8;
        Ok(read_bits)
    }

    fn read_char<R>(input: &mut BitReader<R>, codes_to_chars: &CodesToChars) -> Option<u8>
        where R: Read
    {
        let mut code = Code {
            length: 0,
            data: 0,
        };

        while let Ok(data) = input.read_bit() {
            if data {
                let shifted_one = 1 << code.length;
                code.data |= shifted_one;
            }
            code.length += 1;
            if let Some(&ch) = codes_to_chars.get(&code) {
                return Some(ch);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use encoding::bitreader::BitReader;
    use encoding::bitwriter::BitWriter;
    use self::rand::Rng;
    use std::io::{Cursor, BufWriter, Write};
    use super::*;

    #[test]
    fn simple() {
        assert_text("mississippi river");
        assert_text("12");
        assert_text("3");
        assert_text("");
        assert_data(&[3, 0, 3, 4, 0, 5, 31]);
        assert_data(&[2, 1]);
        assert_data(&[66, 65]);
    }

    #[test]
    #[should_panic(expected = "assertion failed: compressed_length == 0 || compressed_length < decompressed_length")]
    fn full_alphabet() {
        let mut input = (0..256).map(|x| x as u8).collect::<Vec<u8>>();
        for _ in 0..5 {
            let mut clone = input.clone();
            input.append(&mut clone);
        }

        let mut rng = rand::thread_rng();
        rng.shuffle(input.as_mut_slice());

        assert_data(input.as_slice());
    }

    quickcheck! {
        fn random_items(text: Vec<u8>) -> bool {
            let input_slice = &text[..];
            let original_length = (input_slice.len() as u64) * 8;

            let mut input = BitReader::new(Cursor::new(input_slice));

            let output: Vec<u8> = vec![];
            let mut output = BitWriter::new(Cursor::new(output));

            let compressed_length = compress(&mut input, &mut output).unwrap();

            let decompressed: Vec<u8> = vec![];
            let mut decompressed = BufWriter::new(decompressed);

            let mut compressed: BitReader<&[u8]> = BitReader::new(
                output.get_ref().get_ref().as_slice());
            let decompressed_length = decompress(&mut compressed, decompressed.by_ref()).unwrap();

            if compressed_length == 0 && decompressed_length == 0 {
                return true;
            }

            let valid_decompressed_length = original_length == decompressed_length;
            let valid_decompressed_data = input_slice == decompressed.get_ref().as_slice();
            let valid_compressed_length = compressed_length == 0 ||
                                          compressed_length < decompressed_length;

            valid_decompressed_length && valid_decompressed_data && valid_compressed_length
        }
    }

    fn assert_data(input_slice: &[u8]) {
        let original_length = (input_slice.len() as u64) * 8;
        let mut input = BitReader::new(Cursor::new(input_slice));

        let output: Vec<u8> = vec![];
        let mut output = BitWriter::new(Cursor::new(output));

        let compressed_length = compress(&mut input, &mut output).unwrap();

        let decompressed: Vec<u8> = vec![];
        let mut decompressed = BufWriter::new(decompressed);

        let mut compressed: BitReader<&[u8]> =
            BitReader::new(output.get_ref().get_ref().as_slice());
        let decompressed_length = decompress(&mut compressed, decompressed.by_ref()).unwrap();

        assert_eq!(original_length, decompressed_length);
        assert_eq!(input_slice, decompressed.get_ref().as_slice());
        assert!(compressed_length == 0 || compressed_length < decompressed_length);

        // let savings = 1.0 - (compressed_length as f64) / (original_length as f64);
        // println!("savings = {:.2}%% ; compressed_length = {}",
        //          savings * 100.0,
        //          compressed_length);
    }

    fn assert_text(text: &str) {
        assert_data(text.as_bytes());
    }

    #[test]
    fn compute_leaves() {
        let text = "mississippi river";
        let input_slice = text.as_bytes();
        let input = Cursor::new(input_slice);
        let mut input = BitReader::new(input);

        let expect = vec![(' ', 1), ('e', 1), ('i', 5), ('m', 1), ('p', 2), ('r', 2), ('s', 4),
                          ('v', 1)];
        let expect = expect.into_iter()
            .map(|(ch, weight)| {
                NodeData {
                    chars: hashset!{ch as u8},
                    weight: weight,
                }
            })
            .collect::<Vec<NodeData>>();

        let mut input_bytes_read = 0;

        let mut result: Vec<NodeData> = super::compression::compute_leaves(&mut input,
                                                                           &mut input_bytes_read)
            .iter()
            .map(|tree| tree.data().unwrap().clone())
            .collect::<Vec<NodeData>>();
        result.sort_by_key(|node| *node.chars.iter().next().unwrap());

        assert_eq!(input_slice.len() as u64, input_bytes_read);
        assert_eq!(expect, result);
    }

    #[test]
    fn build_tree() {
        use std::collections::HashSet;
        let text = "mississippi river";
        let input_slice = text.as_bytes();
        let input = Cursor::new(input_slice);
        let mut input = BitReader::new(input);
        let mut input_bytes_read = 0;
        let tree = super::compression::build_tree(&mut input, &mut input_bytes_read);
        assert_eq!(input_slice.len() as u64, input_bytes_read);

        let assert_weight = |expect: u64, tree: &Tree| {
            assert_eq!(expect, tree.data().unwrap().weight);
        };

        let mut all_chars = HashSet::with_capacity(input_slice.len());
        for &i in input_slice {
            all_chars.insert(i);
        }

        assert_eq!(all_chars, tree.data().unwrap().chars);
        assert_weight(17, &tree);
        assert_weight(11, &tree.left());
        assert_weight(5, &tree.left().left());
        assert!(tree.left().left().is_leaf());
        assert_weight(6, &tree.left().right());
        assert_weight(2, &tree.left().right().left());
        assert!(tree.left().right().left().is_leaf());
        assert_weight(4, &tree.left().right().right());
        assert!(tree.left().right().right().is_leaf());
        assert_weight(6, &tree.right());
        assert_weight(2, &tree.right().left());
        assert_weight(1, &tree.right().left().left());
        assert_weight(1, &tree.right().left().right());
        assert!(tree.right().left().left().is_leaf());
        assert!(tree.right().left().right().is_leaf());
        assert_weight(4, &tree.right().right());
        assert_weight(1, &tree.right().left().left());
        assert_weight(1, &tree.right().left().right());
        assert_weight(2, &tree.right().right().left());
        assert!(tree.right().right().left().is_leaf());
        assert_weight(2, &tree.right().right().right());
        assert_weight(1, &tree.right().right().right().left());
        assert_weight(1, &tree.right().right().right().right());
        assert!(tree.right().right().right().left().is_leaf());
        assert!(tree.right().right().right().right().is_leaf());
    }

    #[test]
    fn build_dictionary() {
        let text = "mississippi river";
        let input_slice = text.as_bytes();
        let input = Cursor::new(input_slice);
        let mut input = BitReader::new(input);
        let mut input_bytes_read = 0;
        let tree = super::compression::build_tree(&mut input, &mut input_bytes_read);
        let chars_to_codes = super::compression::build_dictionary(&tree);
        for (&ch_a, code_a) in &chars_to_codes {
            for (&ch_b, code_b) in &chars_to_codes {
                assert!(ch_a == ch_b || code_a != code_b);
            }
        }
    }
}
