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
        let mut text_hasher = RollingHash::with_capacity(m, text);
        let pattern_hasher = RollingHash::from(pattern);

        for i in 0..n {
            text_hasher.append();

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

struct RollingHash<'a> {
    hash: u64,
    capacity: usize,

    text: &'a str,
    tail: usize,
}

const BIG_PRIME: u64 = 524_287;
const RADIX: u64 = 256;

impl<'a> RollingHash<'a> {
    fn with_capacity(capacity: usize, text: &'a str) -> Self {
        RollingHash {
            hash: 0,
            capacity: capacity,
            text: text,
            tail: 0,
        }
    }

    fn from(text: &'a str) -> Self {
        let capacity = text.chars().count();
        let mut result = Self::with_capacity(capacity, text);

        for _ in text.chars() {
            result.append();
        }

        result
    }

    fn len(&self) -> usize {
        if self.tail < self.capacity {
            self.tail
        } else {
            self.capacity
        }
    }

    fn hash(&self) -> u64 {
        self.hash
    }

    fn append(&mut self) {
        let tail_item = self.text
            .chars()
            .nth(self.tail)
            .unwrap() as u64;

        if self.len() == self.capacity {
            self.remove_head();
        }

        self.hash = (self.hash * RADIX + tail_item) % BIG_PRIME;
        self.tail += 1;
    }

    fn remove_head(&mut self) {
        let head = if self.tail <= self.capacity {
            0
        } else {
            self.tail - self.capacity
        };

        let head_item = self.text
            .chars()
            .nth(head)
            .unwrap() as u64;

        let radix_pow = self.pow_mod(RADIX, self.capacity as u64 - 1);
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

        let text = "123";
        let text_hash = RollingHash::from(text);

        {
            let mut pattern = RollingHash::with_capacity(text_hash.len(), text);

            pattern.append();
            pattern.append();
            pattern.append();

            assert_eq!(text_hash.hash(), pattern.hash());
        }

        {
            let text_longer = "01234";
            let mut pattern = RollingHash::with_capacity(text_hash.len(), text_longer);

            pattern.append();
            assert!(text_hash.hash() != pattern.hash());

            pattern.append();
            assert!(text_hash.hash() != pattern.hash());

            pattern.append();
            assert!(text_hash.hash() != pattern.hash());

            pattern.append();
            assert_eq!(text_hash.hash(), pattern.hash());

            pattern.append();
            assert!(text_hash.hash() != pattern.hash());
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
