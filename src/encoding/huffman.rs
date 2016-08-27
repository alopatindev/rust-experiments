use std::collections::{HashMap, HashSet};
use std::io::{Read, Result, Write};
use structs::binary_tree::BinaryTree;

pub type NodeData = (HashSet<u8>, usize);
pub type Tree = BinaryTree<NodeData>;

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
    for (&ch, weight) in &char_to_weight {
        let chars = hashset!{*ch};
        let data: NodeData = (chars, weight.clone());
        result.push(BinaryTree::new_leaf(data));
    }

    result
}

// fn make_next_level(level: &Vec<Tree>, nextLevel: &Vec<Tree>) -> (Vec<Tree>, Vec<Tree>) {}

fn build_tree(chars: &[u8]) {
    // let mut leaves = compute_leaves(chars);
    // leaves.as_mut_slice().sort_by(|tree| -tree.data().unwrap().0);
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

        let mut expected: Vec<NodeData> = vec![(hashset!{'e' as u8}, 1),
                                               (hashset!{'s' as u8}, 4),
                                               (hashset!{'m' as u8}, 1),
                                               (hashset!{'i' as u8}, 5),
                                               (hashset!{' ' as u8}, 1),
                                               (hashset!{'v' as u8}, 1),
                                               (hashset!{'p' as u8}, 2),
                                               (hashset!{'r' as u8}, 2)];
        expected.sort_by_key(|node| node.0.iter().next().unwrap().clone());

        let mut result: Vec<NodeData> = super::compute_leaves(input_slice)
            .iter()
            .map(|tree| tree.data().unwrap().clone())
            .collect::<Vec<NodeData>>();
        result.sort_by_key(|node| node.0.iter().next().unwrap().clone());

        assert_eq!(expected, result);
    }
}
