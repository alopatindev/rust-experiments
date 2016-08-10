use std::io::Read;

#[derive(PartialEq, Debug)]
pub struct Counters {
    pub bytes: usize,
    pub characters: usize,
    pub newlines: usize,
    pub words: usize,
}

#[allow(while_let_on_iterator)]
pub fn count(input: &mut Read) -> Counters {
    let mut bytes = 0;
    let mut characters = 0;
    let mut newlines = 0;
    let mut words = 0;

    let is_whitespace = |ch| ch == ' ' || ch == '\n' || ch == '\t';

    let mut word_started = false;
    let mut it = input.chars();

    while let Some(Ok(ch)) = it.next() {
        characters += 1;
        bytes += ch.len_utf8();
        if is_whitespace(ch) {
            if word_started {
                word_started = false;
            }
            if ch == '\n' {
                newlines += 1;
            }
        } else if !word_started {
            word_started = true;
            words += 1;
        }
    }

    Counters {
        bytes: bytes,
        characters: characters,
        newlines: newlines,
        words: words,
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;
    use super::*;

    #[test]
    fn simple() {
        let expect = Counters {
            bytes: 30,
            characters: 20,
            newlines: 2,
            words: 3,
        };
        count_assert(expect, " привет, \n\nраст\t123 ".as_bytes());
    }

    fn count_assert(expect: Counters, input: &[u8]) {
        let mut reader = BufReader::new(input);
        let result = count(&mut reader);
        assert_eq!(expect, result);
    }
}
