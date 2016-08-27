use std::collections::{HashMap, HashSet};
use std::io::{Read, Result, Write};
use structs::binary_tree::BinaryTree;

type NodeData = (HashSet<u8>, usize);
type Tree = BinaryTree<NodeData>;

pub fn compress(input: &mut Read, output: &mut Write) -> Result<usize> {
    unimplemented!();
}

pub fn decompress(input: &mut Read, output: &mut Write) -> Result<usize> {
    unimplemented!();
}

fn compute_leaves(chars: &[u8]) -> Vec<Tree> {
    let mut char_to_weight = HashMap::new();
    for i in chars {
        char_to_weight.entry(i).or_insert(0);
        char_to_weight.get_mut(i).map(|mut i| *i += 1);
    }

    let mut result = Vec::with_capacity(char_to_weight.len());
    for (&ch, &weight) in &char_to_weight {
        let chars = hashset!{*ch};
        let data: NodeData = (chars, weight);
        result.push(BinaryTree::new_leaf(data));
    }

    result
}

fn build_next_level_node(level: &mut Vec<Tree>, next_level: &mut Vec<Tree>) {
    let n = level.len();
    let mut i = 0;
    while i < n {
        let last_node_in_level = i == n - 1;
        let new_parent_has_same_weight = !next_level.is_empty() &&
                                         next_level.last().unwrap().data().unwrap().1 ==
                                         level[i].data().unwrap().1;
        if last_node_in_level || new_parent_has_same_weight {
            let parent = new_parent(&level[i], next_level.last().unwrap());
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
    let ref left_chars = left.data().unwrap().0;
    let ref right_chars = left.data().unwrap().0;

    let mut chars = HashSet::with_capacity(left_chars.len() + right_chars.len());
    chars.clone_from(&left_chars);
    chars.clone_from(&right_chars);

    let weight = left.data().unwrap().1 + right.data().unwrap().1;

    let data = (chars, weight);
    Tree::new(data, left, right)
}

fn build_tree(chars: &[u8]) -> Tree {
    let mut leaves = compute_leaves(chars);
    leaves.sort_by_key(|tree| tree.data().unwrap().1);
    leaves.reverse(); // FIXME

    let mut level = leaves;
    let mut next_level = Vec::with_capacity(level.len() / 2 + 1);

    loop {
        let found_root = next_level.is_empty() && level.len() == 1;
        if found_root {
            return Tree::from_tree(&level[0]);
        } else if level.is_empty() {
            level = next_level;
            next_level = vec![];
        } else {
            build_next_level_node(&mut level, &mut next_level);
        }
    }

    Tree::new_empty()
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
        assert!(decompressed_length == input_slice.len());
    }

    #[test]
    fn leaves() {
        let text = "mississippi river";
        let input_slice = text.as_bytes();

        let expected = vec![(' ', 1), ('e', 1), ('i', 5), ('m', 1), ('p', 2), ('r', 2), ('s', 4),
                            ('v', 1)];
        let expected = expected.into_iter()
            .map(|(ch, weight)| (hashset!{ch as u8}, weight))
            .collect::<Vec<super::NodeData>>();

        let mut result: Vec<super::NodeData> = super::compute_leaves(input_slice)
            .iter()
            .map(|tree| tree.data().unwrap().clone())
            .collect::<Vec<super::NodeData>>();
        result.sort_by_key(|node| *node.0.iter().next().unwrap());

        assert_eq!(expected, result);
    }
}
