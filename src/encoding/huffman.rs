use encoding::bitreader::BitReader;
use encoding::bitwriter::BitWriter;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Read, Result, Seek, SeekFrom, Write};
use structs::binary_tree::BinaryTree;
use structs::bitset::BitSet;

pub struct HuffmanEncoder<W: Write> {
    output: BitWriter<W>,
    char_to_code: HashMap<u8, Code>,
    char_to_weight: HashMap<u8, u64>,
}

pub struct HuffmanDecoder<R: Read + Seek> {
    input: BitReader<R>,
    code_to_char: HashMap<Code, u8>,
}

#[derive(Clone, PartialEq, Debug)]
struct NodeData {
    chars: HashSet<u8>,
    weight: u64,
}

type Tree = BinaryTree<NodeData>;

#[derive(PartialEq, Eq, Hash, Debug)]
struct Code {
    length: u8,
    data: u8,
}

impl<W: Write> HuffmanEncoder<W> {
    pub fn new(output: W) -> Self {
        HuffmanEncoder {
            output: BitWriter::new(output),
            char_to_code: HashMap::with_capacity(256),
            char_to_weight: HashMap::with_capacity(256),
        }
    }

    pub fn analyze<R>(&mut self, input: R) -> Result<u64>
        where R: Read
    {
        let mut input = BitReader::new(input);
        let mut bytes_read = 0;

        while let Ok(buffer) = input.read_u8() {
            self.char_to_weight.entry(buffer).or_insert(0);
            self.char_to_weight.get_mut(&buffer).map(|mut w| *w += 1);
            bytes_read += 1
        }

        Ok(bytes_read * 8)
    }

    pub fn analyze_finish(&mut self) -> Result<()> {
        // TODO: update state
        let leaves = self.compute_leaves();
        let tree = self.build_tree(leaves);
        self.build_dictionary(tree);
        self.write_header()
    }

    pub fn compress<R>(&mut self, input: R) -> Result<u64>
        where R: Read
    {
        let mut input = BitReader::new(input);
        let mut bits_written = 0;

        while let Ok(buffer) = input.read_u8() {
            let code = self.char_to_code.get(&buffer).unwrap();

            for i in 0..code.length {
                let shifted_one = 1 << i;
                let data = (code.data & shifted_one) > 0;
                try!(self.output.write_bit(data));
                bits_written += 1;
            }
        }

        Ok(bits_written)
    }

    pub fn compress_finish(&mut self) {
        self.output.flush();
        // TODO: update state
    }

    fn compute_leaves(&mut self) -> Vec<Tree> {
        let mut leaves: Vec<Tree> = Vec::with_capacity(self.char_to_weight.len());

        for (&ch, &weight) in &self.char_to_weight {
            let data: NodeData = NodeData {
                chars: hashset!{ch},
                weight: weight,
            };
            leaves.push(BinaryTree::new_leaf(data));
        }

        leaves.sort_by_key(|tree| tree.data().unwrap().weight);
        leaves.reverse();
        leaves
    }

    fn build_tree(&self, leaves: Vec<Tree>) -> Tree {
        let mut level = VecDeque::from(leaves);

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
                self.build_next_level(&level, &mut next_level);
                level = next_level;
                next_level = new_level(&level);
            }
        }

        level[0].clone()
    }

    fn build_next_level(&self, level: &VecDeque<Tree>, next_level: &mut VecDeque<Tree>) {
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
                let parent = self.new_parent(&level[i], &head);
                next_level.push_front(parent);
                if last_node_in_level {
                    break;
                }
                k -= 1;
            } else {
                let parent = self.new_parent(&level[i], &level[i - 1]);
                next_level.push_front(parent);
                k -= 2;
            }
        }
    }

    fn new_parent(&self, left: &Tree, right: &Tree) -> Tree {
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

    fn build_dictionary(&mut self, tree: Tree) {
        if let Some(data) = tree.data() {
            for &ch in &data.chars {
                let code = self.compute_code(ch, &tree);
                self.char_to_code.insert(ch, code);
            }
        }

        assert!(self.char_to_code.len() <= 256);
    }

    fn compute_code(&self, ch: u8, tree: &Tree) -> Code {
        let mut tree = tree.clone(); // FIXME

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

    fn write_header(&mut self) -> Result<()> {
        if !self.char_to_code.is_empty() {

            let max_index = (self.char_to_code.len() - 1) as u8;
            try!(self.output.write_u8(max_index));

            for (&ch, code) in &self.char_to_code {
                try!(self.output.write_u8(code.length));
                try!(self.output.write_u8(code.data));
                try!(self.output.write_u8(ch));
            }
        }

        Ok(())
    }

    pub fn position(&self) -> u64 {
        self.output.position()
    }

    pub fn get_output_ref(&self) -> &W {
        self.output.get_ref()
    }
}

impl<R: Read + Seek> HuffmanDecoder<R> {
    pub fn new(input: R) -> Self {
        let mut result = HuffmanDecoder {
            input: BitReader::new(input),
            code_to_char: HashMap::new(),
        };

        if result.read_header().is_err() {
            println!("Failed to read the header");
        }

        result
    }

    pub fn decompress(&mut self,
                      output: &mut Write,
                      offset_bit: u64,
                      original_length_bits: u64)
                      -> Result<u64> {
        let mut read_bytes = 0;

        if original_length_bits == 0 {
            return Ok(read_bytes);
        }

        let original_length_bytes = original_length_bits / 8;

        let offset_byte = offset_bit / 8;
        let _ = try!(self.input.seek(SeekFrom::Start(offset_byte)));
        try!(self.input.skip_bits(offset_bit % 8));

        if original_length_bytes == 1 {
            // FIXME: remove
            assert_eq!(1, self.code_to_char.len());
            let ch = *self.code_to_char.values().next().unwrap();
            try!(output.write_all(&[ch]));
            read_bytes += 1;
        } else {
            while read_bytes < original_length_bytes {
                match self.read_char() {
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


    fn read_header(&mut self) -> Result<()> {
        let len = match self.input.read_u8() {
            Ok(max_index) => (max_index as usize) + 1,
            Err(_) => 0,
        };

        self.code_to_char.reserve(len);

        for _ in 0..len {
            let code_length = try!(self.input.read_u8());
            let code_data = try!(self.input.read_u8());
            let ch = try!(self.input.read_u8());
            let code = Code {
                length: code_length,
                data: code_data,
            };
            self.code_to_char.insert(code, ch);
        }

        Ok(())
    }

    fn read_char(&mut self) -> Option<u8> {
        let mut code = Code {
            length: 0,
            data: 0,
        };

        while let Ok(data) = self.input.read_bit() {
            if data {
                let shifted_one = 1 << code.length;
                code.data |= shifted_one;
            }
            code.length += 1;
            if let Some(&ch) = self.code_to_char.get(&code) {
                return Some(ch);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use self::rand::Rng;
    use std::collections::HashSet;
    use std::io::{Cursor, BufWriter, Write};
    use super::*;
    use super::{NodeData, Tree};

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
            assert_data(&text[..]);
            true
        }
    }

    fn assert_data(input_slice: &[u8]) {
        let mut coder = HuffmanEncoder::new(Cursor::new(vec![]));
        let original_length_bytes = input_slice.len() as u64;
        let original_length_bits = original_length_bytes * 8;
        let analyzed_length_bits = coder.analyze(Cursor::new(input_slice)).unwrap();
        assert_eq!(original_length_bits, analyzed_length_bits);
        let success = coder.analyze_finish().is_ok();
        let data_offset_bit = coder.position();
        assert!(success);

        let compressed_length_bits = coder.compress(Cursor::new(input_slice)).unwrap();
        coder.compress_finish();

        let compressed = Cursor::new(coder.get_output_ref()
            .get_ref()
            .as_slice());
        let mut decompressed = BufWriter::new(vec![]);
        let decompressed_length_bits = HuffmanDecoder::new(compressed)
            .decompress(decompressed.by_ref(), data_offset_bit, original_length_bits)
            .unwrap();

        assert_eq!(original_length_bits, decompressed_length_bits);
        assert_eq!(input_slice, decompressed.get_ref().as_slice());
        assert!(compressed_length_bits <= decompressed_length_bits);

        // let savings = 1.0 - (compressed_length_bits as f64) / (original_length_bits as f64);
        // println!("savings = {:.2}%% ; compressed_length = {}",
        //          savings * 100.0,
        //          compressed_length_bits);
    }

    fn assert_text(text: &str) {
        assert_data(text.as_bytes());
    }

    #[test]
    fn compute_leaves() {
        let text = "mississippi river";
        let input_slice = text.as_bytes();
        let input = Cursor::new(input_slice);

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

        let mut coder = HuffmanEncoder::new(vec![]);
        let _ = coder.analyze(input).unwrap();
        let mut result: Vec<NodeData> = coder.compute_leaves()
            .iter()
            .map(|tree| tree.data().unwrap().clone())
            .collect::<Vec<NodeData>>();
        result.sort_by_key(|node| *node.chars.iter().next().unwrap());

        assert_eq!(expect, result);
    }

    #[test]
    fn build_tree() {
        let text = "mississippi river";
        let input_slice = text.as_bytes();
        let mut coder = HuffmanEncoder::new(vec![]);
        let _ = coder.analyze(Cursor::new(input_slice)).unwrap();

        let leaves = coder.compute_leaves();
        let tree = coder.build_tree(leaves);

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
    fn unique_codes() {
        let text = "mississippi river";
        let input_slice = text.as_bytes();

        let mut coder = HuffmanEncoder::new(vec![]);
        let _ = coder.analyze(Cursor::new(input_slice)).unwrap();
        coder.analyze_finish().unwrap();

        for (&ch_a, code_a) in &coder.char_to_code {
            for (&ch_b, code_b) in &coder.char_to_code {
                assert!(ch_a == ch_b || code_a != code_b);
            }
        }
    }
}
