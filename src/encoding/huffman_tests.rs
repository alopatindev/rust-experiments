#[cfg(test)]
mod tests {
    extern crate rand;

    use rand::Rng;
    use std::collections::HashSet;
    use std::io::{Cursor, Write};
    use super::*;
    use super::{NodeData, Tree};

    const INPUT_TEXT: &'static str = "mississippi river";

    #[test]
    fn single_input() {
        assert_text(INPUT_TEXT);
        assert_text("12");
        assert_text("3");
        assert_text("");
        assert_data(&[3, 0, 3, 4, 0, 5, 31]);
        assert_data(&[2, 1]);
        assert_data(&[66, 65]);
    }

    #[test]
    fn multiple_inputs() {
        let inputs = vec![vec![1, 2]];
        assert!(check_multiple(inputs));

        let inputs = vec![vec![1, 2], vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]];
        assert!(check_multiple(inputs));
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
            check_data(&text[..])
        }

        fn random_multiple(inputs: Vec<Vec<u8>>) -> bool {
            check_multiple(inputs)
        }
    }

    fn check_data(input_slice: &[u8]) -> bool {
        let mut coder = HuffmanEncoder::new(vec![]);
        let original_length_bytes = input_slice.len() as u64;
        let original_length_bits = original_length_bytes * 8;
        let analyzed_length_bits = coder.analyze(input_slice).unwrap();
        assert_eq!(original_length_bits, analyzed_length_bits);
        let success = coder.analyze_finish().is_ok();
        assert!(success);
        let data_offset_bit = coder.position();

        let compressed_length_bits = coder.compress(input_slice).unwrap();
        coder.compress_finish();

        let compressed = Cursor::new(coder.get_output_ref().as_slice());
        let mut decoded = vec![];
        let decoded_length_bits = HuffmanDecoder::new(compressed)
            .unwrap()
            .decode(decoded.by_ref(), data_offset_bit, original_length_bits)
            .unwrap();

        if original_length_bits != decoded_length_bits {
            return false;
        }

        if input_slice != decoded.as_slice() {
            return false;
        }

        if compressed_length_bits > decoded_length_bits {
            return false;
        }

        // let savings = 1.0 - (compressed_length_bits as f64) / (original_length_bits as f64);
        // println!("savings = {:.2}%% ; compressed_length = {}",
        //          savings * 100.0,
        //          compressed_length_bits);

        true
    }

    fn assert_data(data: &[u8]) {
        assert!(check_data(data));
    }

    fn assert_text(text: &str) {
        assert_data(text.as_bytes());
    }

    fn check_multiple(inputs: Vec<Vec<u8>>) -> bool {
        let mut coder = HuffmanEncoder::new(vec![]);

        for i in &inputs {
            let input_slice = i.as_slice();
            let analyzed_length_bits = coder.analyze(input_slice).unwrap();

            let original_length_bytes = input_slice.len() as u64;
            let original_length_bits = original_length_bytes * 8;
            assert_eq!(original_length_bits, analyzed_length_bits);
        }

        let success = coder.analyze_finish().is_ok();
        assert!(success);

        let mut offsets = Vec::with_capacity(inputs.len());
        let mut compressed_lengths = Vec::with_capacity(inputs.len());
        for i in &inputs {
            let input_slice = i.as_slice();
            let data_offset_bit = coder.position();
            offsets.push(data_offset_bit);

            let compressed_length_bits = coder.compress(input_slice).unwrap();
            compressed_lengths.push(compressed_length_bits);
        }

        coder.compress_finish();

        let compressed = Cursor::new(coder.get_output_ref().as_slice());
        let mut decoder = HuffmanDecoder::new(compressed).unwrap();

        for i in 0..inputs.len() {
            let original_length_bytes = inputs[i].len();
            let original_length_bits = original_length_bytes * 8;
            let data_offset_bit = offsets[i];

            let mut decoded = Vec::with_capacity(original_length_bytes);

            let original_length_bits = original_length_bits as u64;
            let decoded_length_bits =
                decoder.decode(&mut decoded, data_offset_bit, original_length_bits)
                    .unwrap();

            if original_length_bits != decoded_length_bits {
                return false;
            }

            if inputs[i].as_slice() != decoded.as_slice() {
                return false;
            }
        }

        true
    }

    #[test]
    fn compute_leaves() {
        let text = INPUT_TEXT;
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
        let text = INPUT_TEXT;
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
        let text = INPUT_TEXT;
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

    #[test]
    #[should_panic]
    fn analyze_after_finish() {
        let text = INPUT_TEXT;
        let input_slice = text.as_bytes();

        let mut coder = HuffmanEncoder::new(vec![]);
        let _ = coder.analyze(Cursor::new(input_slice)).unwrap();
        coder.analyze_finish().unwrap();
        let _ = coder.analyze(Cursor::new(input_slice)).unwrap();
    }

    #[test]
    #[should_panic]
    fn compress_after_finish() {
        let text = INPUT_TEXT;
        let input_slice = text.as_bytes();

        let mut coder = HuffmanEncoder::new(vec![]);
        let _ = coder.analyze(Cursor::new(input_slice)).unwrap();
        coder.analyze_finish().unwrap();

        let _ = coder.compress(Cursor::new(input_slice)).unwrap();
        coder.compress_finish();
        let _ = coder.compress(Cursor::new(input_slice)).unwrap();
    }

    #[test]
    #[should_panic]
    fn analyze_after_compress() {
        let text = INPUT_TEXT;
        let input_slice = text.as_bytes();

        let mut coder = HuffmanEncoder::new(vec![]);
        let _ = coder.analyze(Cursor::new(input_slice)).unwrap();
        coder.analyze_finish().unwrap();

        let _ = coder.compress(Cursor::new(input_slice)).unwrap();
        let _ = coder.analyze(Cursor::new(input_slice)).unwrap();
    }
}
