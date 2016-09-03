use std::io::{Read, Write};

const BUFFER_SIZE: usize = 4096;
const FLUSH_AFTER_LINE: usize = 25;

pub fn head(input: &mut Read, output: &mut Write, limit: usize) {
    let mut buffer = [0; BUFFER_SIZE];
    let mut line = 0;
    let mut ends_with_new_line = false;
    let mut limit_reached;

    loop {
        limit_reached = line >= limit;
        if limit_reached {
            break;
        }

        match input.read(&mut buffer) {
            Err(message) => {
                println!("Error: {}", message);
                break;
            }
            Ok(size) if size == 0 => break,
            Ok(size) => {
                for ch in &buffer[0..size] {
                    output.write(&[*ch]).unwrap();

                    if *ch == b'\n' {
                        line += 1;
                        ends_with_new_line = true;
                        if line % FLUSH_AFTER_LINE == 0 {
                            output.flush().unwrap();
                        }
                    } else {
                        ends_with_new_line = false;
                    }

                    limit_reached = line >= limit;
                    if limit_reached {
                        break;
                    }
                }
            }
        }
    }

    let mut dirty = false;

    if !ends_with_new_line {
        output.write(&[b'\n']).unwrap();
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
    use std::io::{BufReader, BufWriter, Write};
    use super::*;

    #[test]
    fn simple() {
        head_assert("a\nb\n", "a\nb\nc\nd\n", 2);
        head_assert("a\nb\nc\nd\n", "a\nb\nc\nd\n", 200);
        head_assert("a\n", "a\nb\nc\nd\n", 1);
        head_assert("\n", "a\nb\nc\nd\n", 0);
        head_assert("\n", "", 10);
        head_assert("a\nb\nc\nd\n", "a\nb\nc\nd\n", 4);
        head_assert("a\nb\nc\nd\n", "a\nb\nc\nd\n", 5);
        head_assert("\na\nb\nc\n", "\na\nb\nc\nd\n", 4);
        head_assert("hello\nworld\n", "hello\nworld\nfoo\nbar\n", 2);
        head_assert("hello\n\n", "hello\n\nworld\nfoo\nbar\n", 2);
        head_assert("hello\n\nworld\n", "hello\n\nworld\nfoo\nbar\n", 3);
        head_assert("a\n\nb\n\nc\n\nd\n\n\n", "a\n\nb\n\nc\n\nd\n\n\n", 10);
        head_assert("foo\n", "foo", 1);
    }

    fn head_assert(expect: &str, text: &str, limit: usize) {
        let input = text.as_bytes();
        let mut input = BufReader::new(input);

        let output: Vec<u8> = vec![];
        let mut output = BufWriter::new(output);

        head(&mut input, output.by_ref(), limit);
        assert_eq!(expect.as_bytes(), output.get_ref().as_slice());
    }
}
