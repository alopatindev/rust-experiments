use std::collections::VecDeque;

// O(|text| * |pattern|)
pub fn slow_substring_matches(text: &str, pattern: &str) -> bool {
    let n = text.len();
    let m = pattern.len();

    if n >= m {
        for i in 0..(n - m + 1) {
            if string_matches(text, pattern, i) {
                return true;
            }
        }
    }

    false
}

// O(|text|)
// https://www.youtube.com/watch?v=H4VrKHVG5qI
// https://www.youtube.com/watch?v=BRO7mVIFt08&t=36m04s
pub fn karp_rabin_substring_matches(text: &str, pattern: &str) -> bool {
    let n = text.len();
    let m = pattern.len();

    if m == 0 {
        return true;
    } else if n >= m {
        let mut text_hasher = RollingHash::new();
        text_hasher.reserve(m);

        let pattern_hasher = RollingHash::from(pattern);

        for (i, ch) in text.char_indices() {
            if text_hasher.len() == m {
                text_hasher.pop();
            }

            text_hasher.append(ch);

            if text_hasher.len() == m && text_hasher.hash() == pattern_hasher.hash() &&
               string_matches(text, pattern, i + 1 - m) {
                return true;
            }
        }
    }

    false
}

fn string_matches(text: &str, pattern: &str, from: usize) -> bool {
    let m = pattern.len();
    &text[from..(from + m)] == pattern
}

struct RollingHash {
    hash: u64,
    items: VecDeque<u64>,
}

const BASE_PRIME: u64 = 3;

impl RollingHash {
    fn new() -> Self {
        RollingHash {
            hash: 0,
            items: VecDeque::new(),
        }
    }

    fn from(text: &str) -> Self {
        let mut result = Self::new();
        result.reserve(text.len());

        for t in text.chars() {
            result.append(t);
        }

        result
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn hash(&self) -> u64 {
        self.hash
    }

    fn reserve(&mut self, n: usize) {
        self.items.reserve(n);
    }

    fn append(&mut self, ch: char) {
        let m = self.len() as u32;
        let item = ch as u64;
        self.hash += item * BASE_PRIME.pow(m);
        self.items.push_back(item);
    }

    fn pop(&mut self) {
        assert!(self.items.len() > 0);
        if let Some(item) = self.items.pop_front() {
            self.hash = (self.hash - item) / BASE_PRIME;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_slow() {
        simple(&slow_substring_matches);
    }

    #[test]
    fn simple_karp_rabin() {
        simple(&karp_rabin_substring_matches);
    }

    // TODO: random input test
    // TODO: benchmark

    fn simple(substring_matches: &Fn(&str, &str) -> bool) {
        assert!(substring_matches("abc", "bc"));
        assert!(substring_matches("foo bar hello world bla bla bla", "hello world"));
        assert!(substring_matches("hello world bla bla bla", "hello world"));
        assert!(substring_matches("foo bar hello world", "hello world"));
        assert!(substring_matches("hello world", "hello world"));
        assert!(!substring_matches("foo bar hello worl", "hello world"));
        assert!(!substring_matches("ello world bla bla bla", "hello world"));
        assert!(!substring_matches("", "hello world"));
        assert!(!substring_matches("foo", "hello world"));
        assert!(!substring_matches("foo bar Hello world", "hello world"));
        assert!(substring_matches("foo", ""));
        assert!(substring_matches("zzzzzzzzzzzzz hello world zzzzzzz", " zzzzzzz"));
    }
}
