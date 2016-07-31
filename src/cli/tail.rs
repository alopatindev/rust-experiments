use std::io::{Read, Write};
use std::collections::VecDeque;

const FLUSH_AFTER_LINE: usize = 25;

pub fn tail(input: &mut Read, output: &mut Write, limit: usize) {
    let mut ring: VecDeque<Vec<u8>> = VecDeque::with_capacity(limit);
    let mut buffer = [0; 1];
    let mut line_text: Vec<u8> = vec![];

    loop {
        match input.read(&mut buffer) {
            Err(_) => { break }
            Ok(size) if size == 0 => { break }
            Ok(_) => {
                line_text.push(buffer[0]);

                if buffer[0] == b'\n' {
                    ring.push_back(line_text);
                    line_text = vec![];
                    if ring.len() > limit {
                        ring.pop_front();
                    }
                }
            }
        }
    }

    if !line_text.is_empty() || ring.is_empty() {
        line_text.push(b'\n');
        output.write(&line_text[..]).unwrap();
        if ring.len() > limit {
            ring.pop_front();
        }
        output.flush().unwrap();
    } else {
        let mut line = 0;
        for line_text in ring {
            output.write(&line_text[..]).unwrap();

            line += 1;
            if line % FLUSH_AFTER_LINE == 0 {
                output.flush().unwrap();
            }
        }

        if line % FLUSH_AFTER_LINE != 0 {
            output.flush().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, BufWriter, Read, Write};
    use super::*;

    #[test]
    fn test_tail() {
        test_tail_with("c\nd\n", "a\nb\nc\nd\n", 2);
        test_tail_with("a\nb\nc\nd\n", "a\nb\nc\nd\n", 200);
        test_tail_with("d\n", "a\nb\nc\nd\n", 1);
        test_tail_with("\n", "a\nb\nc\nd\n", 0);
        test_tail_with("\n", "", 10);
        test_tail_with("a\nb\nc\nd\n", "a\nb\nc\nd\n", 4);
        test_tail_with("a\nb\nc\nd\n", "a\nb\nc\nd\n", 5);
        test_tail_with("a\nb\nc\nd\n", "\na\nb\nc\nd\n", 4);
        test_tail_with("b\nc\nd\n\n", "\na\nb\nc\nd\n\n", 4);
        test_tail_with("foo\nbar\n", "hello\nworld\nfoo\nbar\n", 2);
        test_tail_with("foo\nbar\n", "hello\n\nworld\nfoo\nbar\n", 2);
        test_tail_with("foo\n\nbar\n", "hello\n\nworld\nfoo\n\nbar\n", 3);
        test_tail_with("a\n\nb\n\nc\n\nd\n\n\n", "a\n\nb\n\nc\n\nd\n\n\n", 10);
        test_tail_with("foo\n", "foo", 1);
    }

    fn test_tail_with(expect: &str, text: &str, limit: usize) {
        let input_take = text.as_bytes().take(text.len() as u64);
        let mut input = BufReader::new(input_take);

        let output_vec: Vec<u8> = vec![];
        let mut output = BufWriter::new(output_vec);

        tail(&mut input, output.by_ref(), limit);
        assert_eq!(expect.as_bytes(), &output.get_ref()[..]);
    }
}
