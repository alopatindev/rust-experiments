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
// https://www.youtube.com/watch?v=RkBrAXoau8Y
// https://www.youtube.com/watch?v=BRO7mVIFt08&t=36m04s
pub fn karp_rabin_substring_find(text: &str, pattern: &str) -> Option<usize> {
    let n = text.chars().count();
    let m = pattern.chars().count();

    if m == 0 {
        return Some(0);
    } else if n >= m {
        let mut text_hasher = RollingHash::with_capacity(m);
        let pattern_hasher = RollingHash::from(pattern);

        for (i, ch) in text.chars().enumerate() {
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
    capacity: usize,
}

const BIG_PRIME: u64 = 524_287;
const RADIX: u64 = 256;

impl RollingHash {
    fn with_capacity(capacity: usize) -> Self {
        RollingHash {
            hash: 0,
            items: VecDeque::with_capacity(capacity),
            capacity: capacity,
        }
    }

    fn from(text: &str) -> Self {
        let capacity = text.chars().count();
        let mut result = Self::with_capacity(capacity);

        for ch in text.chars() {
            result.append(ch);
        }

        result
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn hash(&self) -> u64 {
        self.hash
    }

    fn append(&mut self, ch: char) {
        let item = ch as u64;

        if self.len() == self.capacity {
            self.remove_head();
        }

        self.hash = (self.hash * RADIX + item) % BIG_PRIME;
        self.items.push_back(item);
    }

    fn remove_head(&mut self) {
        let radix_pow = self.pow_mod(RADIX, self.capacity as u64 - 1);
        let head_item = self.items.pop_front().unwrap();
        let head_hash = (radix_pow * head_item) % BIG_PRIME;

        self.hash = self.hash + BIG_PRIME - head_hash;
        self.hash %= BIG_PRIME;
    }

    fn pow_mod(&self, x: u64, exp: u64) -> u64 {
        let mut result = 1;

        for _ in 0..exp {
            result = (result * x) % BIG_PRIME;
        }

        result
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

    quickcheck! {
        fn random_slow(text: String, pattern: String) -> bool {
            random_check(&slow_substring_find, text, pattern)
        }

        fn random_karp_rabin(text: String, pattern: String) -> bool {
            random_check(&karp_rabin_substring_find, text, pattern)
        }
    }

    // TODO: benchmark

    #[test]
    fn test_rolling_hash() {
        use super::RollingHash;

        let text = RollingHash::from("123");

        {
            let mut pattern = RollingHash::with_capacity(text.len());

            pattern.append('1');
            pattern.append('2');
            pattern.append('3');

            assert_eq!(text.hash(), pattern.hash());
        }

        {
            let mut pattern = RollingHash::with_capacity(text.len());

            pattern.append('0');
            pattern.append('1');
            pattern.append('2');
            assert!(text.hash() != pattern.hash());

            pattern.append('3');
            assert_eq!(text.hash(), pattern.hash());
        }
    }

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
        assert_eq!(Some(13),
                   substring_find("1111 2342342 abcdef abcdef hello world banana
 foo bar hello world banananana bananananananana foo bar hello world asdfghjk
 asdf asdf sadf asdf sadf asdf sadf hello",
                                  "abcdef abcdef hello world banana
 foo bar hello world banananana bananananananana foo bar hello world asdfghjk
 asdf asdf sadf asdf sadf asdf sadf hello"));
    }

    fn random_check(substring_find: &Fn(&str, &str) -> Option<usize>,
                    text: String,
                    pattern: String)
                    -> bool {
        let text = text.as_str();
        let pattern = pattern.as_str();

        match substring_find(text, pattern) {
            Some(pos) => {
                let pattern_found: String = text.chars()
                    .skip(pos)
                    .take(pattern.chars().count())
                    .collect();
                pattern == pattern_found.as_str()
            }
            None => true,
        }
    }
}
