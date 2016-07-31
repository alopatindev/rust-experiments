use std::io::{Read, Write};

const FLUSH_AFTER_LINE: usize = 25;

pub fn head(input: &mut Read, output: &mut Write, limit: usize) {
    let mut buffer = [0; 1];
    let mut line = 0;

    loop {
        let limit_reached = line >= limit;
        if limit_reached { break }

        match input.read(&mut buffer) {
            Err(_) => break,
            Ok(size) if size == 0 => break,
            Ok(_) => {
                output.write(&buffer).unwrap();

                if buffer[0] == b'\n' {
                    line += 1;
                    if line % FLUSH_AFTER_LINE == 0 {
                        output.flush().unwrap();
                    }
                    continue
                }
            }
        }
    }

    let mut dirty = false;

    if buffer[0] != b'\n' {
        buffer[0] = b'\n';
        output.write(&buffer).unwrap();
        dirty = true;
    } else if line % FLUSH_AFTER_LINE != 0 {
        dirty = true;
    }

    if dirty {
        output.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, BufWriter, Read, Write};
    use super::*;

    #[test]
    fn test_head() {
        test_head_with("a\nb\n", "a\nb\nc\nd\n", 2);
        test_head_with("a\nb\nc\nd\n", "a\nb\nc\nd\n", 200);
        test_head_with("a\n", "a\nb\nc\nd\n", 1);
        test_head_with("\n", "a\nb\nc\nd\n", 0);
        test_head_with("\n", "", 10);
        test_head_with("a\nb\nc\nd\n", "a\nb\nc\nd\n", 4);
        test_head_with("a\nb\nc\nd\n", "a\nb\nc\nd\n", 5);
        test_head_with("\na\nb\nc\n", "\na\nb\nc\nd\n", 4);
        test_head_with("hello\nworld\n", "hello\nworld\nfoo\nbar\n", 2);
        test_head_with("hello\n\n", "hello\n\nworld\nfoo\nbar\n", 2);
        test_head_with("hello\n\nworld\n", "hello\n\nworld\nfoo\nbar\n", 3);
        test_head_with("a\n\nb\n\nc\n\nd\n\n\n", "a\n\nb\n\nc\n\nd\n\n\n", 10);
        test_head_with("foo\n", "foo", 1);
    }

    fn test_head_with(expect: &str, text: &str, limit: usize) {
        let input_take = text.as_bytes().take(text.len() as u64);
        let mut input = BufReader::new(input_take);

        let output_vec: Vec<u8> = vec![];
        let mut output = BufWriter::new(output_vec);

        head(&mut input, output.by_ref(), limit);
        assert_eq!(expect.as_bytes(), &output.get_ref()[..]);
    }
}
