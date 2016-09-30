pub struct HuffmanEncoder<W: Write> {
    state: State,
    output: BitWriter<W>,
    char_to_code: HashMap<Char, Code>,
    char_to_weight: HashMap<Char, u64>,
}

fn max_possible_chars() -> usize {
    1 << (mem::size_of::<Char>() * 8)
}

fn max_code_length() -> CodeLength {
    let size_bits = mem::size_of::<CodeData>() * 8;
    size_bits as CodeLength - 1
}

impl<W: Write> HuffmanEncoder<W> {
    pub fn new(output: W) -> Self {
        HuffmanEncoder {
            state: State::Initial,
            output: BitWriter::new(output),
            char_to_code: HashMap::with_capacity(max_possible_chars()),
            char_to_weight: HashMap::with_capacity(max_possible_chars()),
        }
    }

    pub fn analyze<R>(&mut self, input: R) -> Result<u64>
        where R: Read
    {
        assert_eq!(State::Initial, self.state);

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
        assert_eq!(State::Initial, self.state);
        self.state = State::Analyzed;

        let leaves = self.compute_leaves();
        let tree = self.build_tree(leaves);
        self.build_dictionary(tree);
        self.write_header()
    }

    pub fn compress<R>(&mut self, input: R) -> Result<u64>
        where R: Read
    {
        assert_eq!(State::Analyzed, self.state);

        let mut input = BitReader::new(input);
        let mut bits_written = 0;

        while let Ok(buffer) = input.read_u8() {
            let code = self.char_to_code.get(&buffer).unwrap();

            assert!(code.length <= max_code_length());
            for i in 0..code.length {
                let shifted_one = 1 << i;
                let data = (code.data & shifted_one) > 0;
                try!(self.output.write_bit(data));
                bits_written += 1;
            }
        }

        Ok(bits_written)
    }

    pub fn compress_finish(&mut self) -> Result<()> {
        assert_eq!(State::Analyzed, self.state);
        self.state = State::Compressed;

        self.output.flush()
    }

    pub fn position(&self) -> u64 {
        self.output.position()
    }

    pub fn get_output_ref(&self) -> &W {
        self.output.get_ref()
    }

    pub fn get_output_mut(&mut self) -> &mut W {
        self.output.get_mut()
    }

    pub fn get_writer_mut(&mut self) -> &mut BitWriter<W> {
        &mut self.output
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
        let mut level_length = n;

        while level_length > 0 {
            let i = level_length - 1;
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
                level_length -= 1;
            } else {
                let parent = self.new_parent(&level[i], &level[i - 1]);
                next_level.push_front(parent);
                level_length -= 2;
            }
        }
    }

    fn new_parent(&self, left: &Tree, right: &Tree) -> Tree {
        let left_chars = &left.data().unwrap().chars;
        let right_chars = &right.data().unwrap().chars;

        let chars = left_chars.union(right_chars).cloned().collect::<HashSet<Char>>();
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

        assert!(self.char_to_code.len() <= max_possible_chars());
    }

    fn compute_code(&self, ch: Char, tree: &Tree) -> Code {
        let mut tree = tree.clone();
        let mut code = BitSet::new();
        let mut length: CodeLength = 0;

        loop {
            if tree.left_data().is_some() && tree.left_data().unwrap().chars.contains(&ch) {
                tree = tree.left();
            } else if tree.right_data().is_some() &&
                      tree.right_data().unwrap().chars.contains(&ch) {
                code.insert(length as usize);
                tree = tree.right();
            } else {
                break;
            }
            length += 1;
        }

        assert!(tree.is_leaf());

        if length == 0 {
            length = 1; // FIXME
        }

        assert!(length > 0);
        assert!(length <= max_code_length());

        let data = code.as_slice()[0] as CodeData;

        Code {
            length: length,
            data: data,
        }
    }

    fn write_header(&mut self) -> Result<()> {
        let dict_length = self.char_to_code.len() as DictLength;
        try!(self.output.write_u16(dict_length));

        for (&ch, code) in &self.char_to_code {
            try!(self.output.write_u8(code.length));
            try!(self.output.write_u16(code.data));
            try!(self.output.write_u8(ch));
        }

        Ok(())
    }
}

impl<W: Write> Drop for HuffmanEncoder<W> {
    fn drop(&mut self) {
        if self.state == State::Analyzed {
            let _ = self.compress_finish();
        }
    }
}
