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
