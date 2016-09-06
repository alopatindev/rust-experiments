pub struct HuffmanEncoder<W: Write> {
    output: BitWriter<W>,
    char_to_code: HashMap<u8, Code>,
    char_to_weight: HashMap<u8, u64>,
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
