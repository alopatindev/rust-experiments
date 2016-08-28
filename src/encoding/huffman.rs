use std::collections::{HashMap, HashSet};
use std::io::{BufReader, Read, Result, Write};
use structs::binary_tree::BinaryTree;

type NodeData = (HashSet<u8>, usize);
type Tree = BinaryTree<NodeData>;

const BUFFER_SIZE: usize = 4096;

pub fn compress<T>(input: &mut BufReader<T>, output: &mut Write) -> Result<usize>
    where T: Read
{
    let tree = build_tree(input);
    unimplemented!();
}

pub fn decompress(input: &mut Read, output: &mut Write) -> Result<usize> {
    unimplemented!();
}

fn compute_leaves<T>(input: &mut BufReader<T>) -> Vec<Tree>
    where T: Read
{
    let mut char_to_weight: HashMap<u8, usize> = HashMap::new();
    let mut buffer = [0; BUFFER_SIZE];

    loop {
        let bytes_read = input.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }

        for i in &buffer[0..bytes_read] {
            char_to_weight.entry(*i).or_insert(0);
            char_to_weight.get_mut(i).map(|mut w| *w += 1);
        }
    }

    let mut result = Vec::with_capacity(char_to_weight.len());
    for (&ch, &weight) in &char_to_weight {
        let chars = hashset!{ch};
        let data: NodeData = (chars, weight);
        result.push(BinaryTree::new_leaf(data));
    }

    result
}

fn build_next_level(level: &[Tree], next_level: &mut Vec<Tree>) {
    let n = level.len();
    let mut i = 0;
    while i < n {
        let last_node_in_level = i == n - 1;
        let new_parent_has_same_weight = match next_level.last() {
            Some(tree) => tree.data().unwrap().1 <= level[i].data().unwrap().1,
            None => false,
        };
        if last_node_in_level || new_parent_has_same_weight {
            let parent = new_parent(next_level.last().unwrap(), &level[i]);
            next_level.pop();
            next_level.push(parent);
            i += 1;
        } else {
            let parent = new_parent(&level[i], &level[i + 1]);
            next_level.push(parent);
            i += 2;
        }
    }
}

fn new_parent(left: &Tree, right: &Tree) -> Tree {
    let left_chars = &left.data().unwrap().0;
    let right_chars = &right.data().unwrap().0;

    let chars = left_chars.union(right_chars).cloned().collect::<HashSet<u8>>();
    let weight = left.data().unwrap().1 + right.data().unwrap().1;

    let data = (chars, weight);
    Tree::new(data, left, right)
}

fn build_tree<T>(chars: &mut BufReader<T>) -> Tree
    where T: Read
{
    let mut leaves = compute_leaves(chars);
    leaves.sort_by_key(|tree| tree.data().unwrap().1);

    let mut level = leaves;
    let mut next_level = Vec::with_capacity(level.len() / 2 + 1);

    loop {
        let found_root = next_level.is_empty() && level.len() == 1;
        if found_root {
            break;
        } else {
            build_next_level(&level, &mut next_level);
            level = next_level;
            next_level = vec![];
        }
    }

    Tree::from_tree(&level[0])
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, BufWriter, Write};
    use structs::binary_tree::BinaryTree;
    use super::*;

    #[test]
    fn simple() {
        simple_assert("mississippi river");
    }

    // TODO: quickcheck

    fn simple_assert(text: &str) {
        let input_slice = text.as_bytes();
        let mut input = BufReader::new(input_slice);

        let output_vec: Vec<u8> = vec![];
        let mut output = BufWriter::new(output_vec);

        let compressed_length = compress(&mut input, output.by_ref()).unwrap();

        let decompressed: Vec<u8> = vec![];
        let mut decompressed = BufWriter::new(decompressed);

        let mut compressed = BufReader::new(&output.get_ref()[..]);
        let decompressed_length = decompress(&mut compressed, decompressed.by_ref()).unwrap();

        assert_eq!(input_slice, &decompressed.get_ref()[..]);
        assert!(compressed_length < decompressed_length);
        assert!(decompressed_length == input_slice.len() * 8);
        assert!(compressed_length == 46);
    }

    #[test]
    fn compute_leaves() {
        let text = "mississippi river";
        let input_slice = text.as_bytes();
        let mut input = BufReader::new(input_slice);

        let expected = vec![(' ', 1), ('e', 1), ('i', 5), ('m', 1), ('p', 2), ('r', 2), ('s', 4),
                            ('v', 1)];
        let expected = expected.into_iter()
            .map(|(ch, weight)| (hashset!{ch as u8}, weight))
            .collect::<Vec<super::NodeData>>();

        let mut result: Vec<super::NodeData> = super::compute_leaves(&mut input)
            .iter()
            .map(|tree| tree.data().unwrap().clone())
            .collect::<Vec<super::NodeData>>();
        result.sort_by_key(|node| *node.0.iter().next().unwrap());

        assert_eq!(expected, result);
    }

    #[test]
    fn build_tree() {
        use std::collections::HashSet;
        let text = "mississippi river";
        let input_slice = text.as_bytes();
        let mut input = BufReader::new(input_slice);
        let tree = super::build_tree(&mut input);

        let assert_weight = |expect: usize, tree: &super::Tree| {
            assert_eq!(expect, tree.data().unwrap().1);
        };

        let mut all_chars = HashSet::with_capacity(input_slice.len());
        for &i in input_slice {
            all_chars.insert(i);
        }

        assert_eq!(all_chars, tree.data().unwrap().0);
        assert_weight(17, &tree);
        assert_weight(6, &tree.left());
        assert_weight(2, &tree.left().left());
        assert_weight(1, &tree.left().left().left());
        assert!(tree.left().left().left().is_leaf());
        assert_weight(1, &tree.left().left().right());
        assert!(tree.left().left().right().is_leaf());
        assert_weight(4, &tree.left().right());
        assert_weight(2, &tree.left().right().left());
        assert_weight(1, &tree.left().right().left().left());
        assert!(tree.left().right().left().left().is_leaf());
        assert_weight(1, &tree.left().right().left().right());
        assert!(tree.left().right().left().right().is_leaf());
        assert_weight(2, &tree.left().right().right());
        assert_weight(11, &tree.right());
        assert_weight(6, &tree.right().left());
        assert_weight(2, &tree.right().left().left());
        assert!(tree.right().left().left().is_leaf());
        assert_weight(4, &tree.right().left().right());
        assert!(tree.right().left().right().is_leaf());
        assert_weight(5, &tree.right().right());
    }
}
