use std::collections::VecDeque;

// O(|text| * |pattern|)
pub fn slow_substring_find(text: &str, pattern: &str) -> Option<usize> {
    let n = text.chars().count();
    let m = pattern.chars().count();

    if n >= m {
        for i in 0..(n - m + 1) {
            if string_find(text, pattern, i) {
                return Some(i);
            }
        }
    }

    None
}

// O(|text|)
// https://www.youtube.com/watch?v=H4VrKHVG5qI
// https://www.youtube.com/watch?v=BRO7mVIFt08&t=36m04s
pub fn karp_rabin_substring_find(text: &str, pattern: &str) -> Option<usize> {
    let n = text.chars().count();
    let m = pattern.chars().count();

    if m == 0 {
        return Some(0);
    } else if n >= m {
        let mut text_hasher = RollingHash::new();
        text_hasher.reserve(m);

        let pattern_hasher = RollingHash::from(pattern);

        for (i, ch) in text.chars().enumerate() {
            if text_hasher.len() == m {
                text_hasher.pop();
            }

            text_hasher.append(ch);

            if text_hasher.len() == m && text_hasher.hash() == pattern_hasher.hash() {
                let from = i + 1 - m;
                if string_find(text, pattern, from) {
                    return Some(from);
                }
            }
        }
    }

    None
}

fn string_find(text: &str, pattern: &str, from: usize) -> bool {
    let m = pattern.chars().count();

    // basically means &text[from..(from + m)] == pattern
    let ts: Vec<char> = text.chars().skip(from).take(m).collect();
    let ps: Vec<char> = pattern.chars().collect();
    ts == ps
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
        result.reserve(text.chars().count());

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
        simple(&slow_substring_find);
    }

    #[test]
    fn simple_karp_rabin() {
        simple(&karp_rabin_substring_find);
    }

    // TODO: random input test
    // TODO: benchmark

    fn simple(substring_find: &Fn(&str, &str) -> Option<usize>) {
        assert_eq!(Some(1), substring_find("abc", "bc"));
        assert_eq!(Some(8),
                   substring_find("foo bar hello world bla bla bla", "hello world"));
        assert_eq!(Some(0),
                   substring_find("hello world bla bla bla", "hello world"));
        assert_eq!(Some(8),
                   substring_find("foo bar hello world", "hello world"));
        assert_eq!(Some(0), substring_find("hello world", "hello world"));
        assert_eq!(None, substring_find("foo bar hello worl", "hello world"));
        assert_eq!(None,
                   substring_find("ello world bla bla bla", "hello world"));
        assert_eq!(None, substring_find("", "hello world"));
        assert_eq!(None, substring_find("foo", "hello world"));
        assert_eq!(None, substring_find("foo bar Hello world", "hello world"));
        assert_eq!(Some(0), substring_find("foo", ""));
        assert_eq!(Some(25),
                   substring_find("zzzzzzzzzzzzz hello world zzzzzzz", " zzzzzzz"));
        assert_eq!(Some(6),
                   substring_find("Абвгд Привет Раст еёжз",
                                  "Привет Раст"));
    }
}
