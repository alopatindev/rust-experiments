use std::collections::VecDeque;
use std::io::{Read, Write};

const BUFFER_SIZE: usize = 4096;
const FLUSH_AFTER_LINE: usize = 25;

pub fn tail(input: &mut Read, output: &mut Write, limit: usize) {
    let mut ring: VecDeque<Vec<u8>> = VecDeque::with_capacity(limit);
    let mut buffer = [0; BUFFER_SIZE];
    let mut line_text: Vec<u8> = vec![];

    loop {
        match input.read(&mut buffer) {
            Err(message) => {
                println!("Error: {}", message);
                break;
            }
            Ok(size) if size == 0 => break,
            Ok(size) => {
                for ch in &buffer[0..size] {
                    line_text.push(*ch);
                    if *ch == b'\n' {
                        ring.push_back(line_text);
                        line_text = vec![];
                        if ring.len() > limit {
                            ring.pop_front();
                        }
                    }
                }
            }
        }
    }

    if !line_text.is_empty() || ring.is_empty() {
        line_text.push(b'\n');
        output.write(&line_text[..]).unwrap();
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
    use std::io::{BufReader, BufWriter, Write};
    use super::*;

    #[test]
    fn simple() {
        tail_assert("c\nd\n", "a\nb\nc\nd\n", 2);
        tail_assert("a\nb\nc\nd\n", "a\nb\nc\nd\n", 200);
        tail_assert("d\n", "a\nb\nc\nd\n", 1);
        tail_assert("\n", "a\nb\nc\nd\n", 0);
        tail_assert("\n", "", 10);
        tail_assert("a\nb\nc\nd\n", "a\nb\nc\nd\n", 4);
        tail_assert("a\nb\nc\nd\n", "a\nb\nc\nd\n", 5);
        tail_assert("a\nb\nc\nd\n", "\na\nb\nc\nd\n", 4);
        tail_assert("b\nc\nd\n\n", "\na\nb\nc\nd\n\n", 4);
        tail_assert("foo\nbar\n", "hello\nworld\nfoo\nbar\n", 2);
        tail_assert("foo\nbar\n", "hello\n\nworld\nfoo\nbar\n", 2);
        tail_assert("foo\n\nbar\n", "hello\n\nworld\nfoo\n\nbar\n", 3);
        tail_assert("a\n\nb\n\nc\n\nd\n\n\n", "a\n\nb\n\nc\n\nd\n\n\n", 10);
        tail_assert("foo\n", "foo", 1);
    }

    fn tail_assert(expect: &str, text: &str, limit: usize) {
        let input = text.as_bytes();
        let mut input = BufReader::new(input);

        let output: Vec<u8> = vec![];
        let mut output = BufWriter::new(output);

        tail(&mut input, output.by_ref(), limit);
        assert_eq!(expect.as_bytes(), output.get_ref().as_slice());
    }
}
