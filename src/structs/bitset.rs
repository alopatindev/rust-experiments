extern crate sys_info;
use std::{fmt, mem};

include!("bitset_iterators.rs");

pub struct BitSet {
    buckets: Vec<usize>,  // 0th bucket is the lowest
    size: usize,
}

impl BitSet {
    pub fn new() -> BitSet {
        BitSet {
            buckets: vec![0; 1],
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn next_set_bit(&self, from_index: usize) -> Option<usize> {
        self.next_bit(from_index, true)
    }

    pub fn next_clear_bit(&self, from_index: usize) -> Option<usize> {
        self.next_bit(from_index, false)
    }

    pub fn previous_set_bit(&self, from_index: usize) -> Option<usize> {
        self.previous_bit(from_index, true)
    }

    pub fn previous_clear_bit(&self, from_index: usize) -> Option<usize> {
        self.previous_bit(from_index, false)
    }

    pub fn insert(&mut self, index: usize) {
        if !self.contains(index) {
            self.maybe_grow_buckets(index);
            let (bucket_index, bit_index) = self.split_index(index);
            let bit = 1usize << bit_index;
            self.buckets[bucket_index] |= bit;
            self.size += 1;
        }
    }

    pub fn remove(&mut self, index: usize) {
        if self.contains(index) {
            let (bucket_index, bit_index) = self.split_index(index);
            let mask = !(1usize << bit_index);
            self.buckets[bucket_index] &= mask;
            self.size -= 1;
        }
    }

    pub fn contains(&self, index: usize) -> bool {
        if index > self.max_index() {
            false
        } else {
            let (bucket_index, bit_index) = self.split_index(index);
            let bit = 1usize << bit_index;
            (self.buckets[bucket_index] & bit) > 0
        }
    }

    pub fn iter(&self) -> BitSetIterator {
        BitSetIterator { set: self, index: 0 }
    }

    fn bucket_size_in_bits(&self) -> usize {
        let bucket_size = mem::size_of::<usize>();
        bucket_size * 8
    }

    fn max_index(&self) -> usize {
        (self.buckets.len() * self.bucket_size_in_bits()) - 1
    }

    fn maybe_grow_buckets(&mut self, index: usize) {
        assert!(self.can_grow_buckets(index), "not enough memory to grow buckets");

        while index > self.max_index() {
            self.buckets.push(0);
        }
    }

    fn can_grow_buckets(&self, index: usize) -> bool {
        if index <= self.max_index() {
            true
        } else {
            let info = sys_info::mem_info();
            match info {
                Ok(info) => {
                    let mem_free = info.free + info.swap_free;
                    let (bucket_index, _) = self.split_index(index);
                    bucket_index < mem_free as usize
                }
                Err(_) => {
                    true
                }
            }
        }
    }

    fn split_index(&self, index: usize) -> (usize, usize) {
        let n = self.max_index() + 1;
        let bucket_index = index / self.bucket_size_in_bits();
        let bit_index = (index % n) % self.bucket_size_in_bits();
        (bucket_index, bit_index)
    }

    fn next_bit(&self, from_index: usize, pattern: bool) -> Option<usize> {
        if from_index < usize::max_value() {
            let pattern = if pattern { 1 } else { 0 };

            let mut index = from_index + 1;
            loop {
                let (bucket_index, bit_index) = self.split_index(index);
                if bucket_index >= self.buckets.len() {
                    break
                }
                let rest_of_bits = self.buckets[bucket_index] >> bit_index;
                let found = (rest_of_bits & 1) == pattern;
                if found {
                    return Some(index)
                } else if index < self.max_index() {
                    index += 1;
                } else {
                    break
                }
            }
        }

        None
    }

    fn previous_bit(&self, from_index: usize, pattern: bool) -> Option<usize> {
        if from_index > 0 {
            let pattern = if pattern { 1 } else { 0 };

            let mut index = from_index - 1;
            loop {
                let (bucket_index, bit_index) = self.split_index(index);
                let rest_of_bits = self.buckets[bucket_index] >> bit_index;
                let found = rest_of_bits & 1 == pattern;
                if found {
                    return Some(index)
                } else if index > 0 {
                    index -= 1;
                } else {
                    break
                }
            }
        }

        None
    }
}

impl fmt::Display for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let items: Vec<String> = self
            .iter()
            .map(|i| i.to_string())
            .collect();
        write!(f, "{{{}}}", items.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const N: usize = 2048;

    #[test]
    fn empty() {
        let b: BitSet = BitSet::new();
        let is_empty_set: bool = (0..N)
            .filter(|i: &usize| { b.contains(*i) })
            .count() == 0;
        assert!(is_empty_set);
    }

    #[test]
    fn single_item() {
        for i in 0..N {
            let mut b: BitSet = BitSet::new();
            b.insert(i);
            let xs: Vec<usize> = (0..N)
                .filter(|j: &usize| b.contains(*j))
                .collect();
            let contains_single_item_only = xs.len() == 1 && xs[0] == i;
            assert!(contains_single_item_only);
        }
    }

    #[test]
    fn into_iter() {
        let a = vec![5,55,63,64,65,70,88];
        let mut b: BitSet = BitSet::new();
        for i in &a {
            b.insert(*i);
        }

        let mut a_iter = a.iter();
        for i in b {
            let j = a_iter.next().unwrap();
            assert_eq!(j, &i);
        }
    }

    #[test]
    fn set_twice() {
        let mut b: BitSet = BitSet::new();
        for i in 0..N {
            b.insert(i);
            assert_eq!(true, b.contains(i));
            b.insert(i);
            assert_eq!(true, b.contains(i));
        }
        assert_eq!(N, b.len());
    }

    #[test]
    fn set_and_clear() {
        let mut b: BitSet = BitSet::new();
        for i in 0..N {
            b.insert(i);
            assert_eq!(true, b.contains(i));
            b.remove(i);
            assert_eq!(false, b.contains(i));
        }
        assert_eq!(0, b.len());

        for i in 0..N {
            b.remove(i);
            assert_eq!(false, b.contains(i));
        }
        assert_eq!(0, b.len());
    }

    #[test]
    fn next_set_bit() {
        let a = vec![5,55,63,64,65,70,88];
        let mut b: BitSet = BitSet::new();
        for i in &a {
            b.insert(*i);
        }

        let mut index = 0;
        for i in &a {
            assert_eq!(*i, b.next_set_bit(index).unwrap());
            index = *i;
        }

        assert_eq!(None, b.next_set_bit(index));
        assert_eq!(None, b.next_set_bit(usize::max_value()));
    }

    #[test]
    fn next_clear_bit() {
        let a = vec![1,2,6,7];
        let mut b: BitSet = BitSet::new();
        for i in &a {
            b.insert(*i);
        }

        assert_eq!(Some(3), b.next_clear_bit(0));
        assert_eq!(Some(3), b.next_clear_bit(1));
        assert_eq!(Some(3), b.next_clear_bit(2));
        assert_eq!(Some(4), b.next_clear_bit(3));
        assert_eq!(Some(5), b.next_clear_bit(4));
        assert_eq!(Some(8), b.next_clear_bit(5));
        assert_eq!(Some(8), b.next_clear_bit(6));
        assert_eq!(Some(8), b.next_clear_bit(7));
        assert_eq!(Some(9), b.next_clear_bit(8));

        b.insert(9);
        assert_eq!(Some(8), b.next_clear_bit(5));
        assert_eq!(Some(8), b.next_clear_bit(6));
        assert_eq!(Some(8), b.next_clear_bit(7));
        assert_eq!(Some(10), b.next_clear_bit(8));

        b.remove(1);
        assert_eq!(Some(1), b.next_clear_bit(0));

        assert_eq!(None, b.next_clear_bit(63));
        assert_eq!(None, b.next_clear_bit(64));

        b.insert(64);
        assert_eq!(Some(65), b.next_clear_bit(63));
        assert_eq!(Some(65), b.next_clear_bit(64));
    }

    #[test]
    fn previous_set_bit() {
        let a = vec![5,55,63,64,65,70,88];
        let mut b: BitSet = BitSet::new();
        for i in &a {
            b.insert(*i);
        }

        let mut index = 88;
        for i in a.iter().rev().skip(1) {
            assert_eq!(*i, b.previous_set_bit(index).unwrap());
            index = *i;
        }

        assert_eq!(None, b.previous_set_bit(index));
    }

    #[test]
    fn previous_clear_bit() {
        let a = vec![1,2,6,7];
        let mut b: BitSet = BitSet::new();
        for i in &a {
            b.insert(*i);
        }

        assert_eq!(Some(8), b.previous_clear_bit(9));
        assert_eq!(Some(5), b.previous_clear_bit(8));
        assert_eq!(Some(5), b.previous_clear_bit(7));
        assert_eq!(Some(5), b.previous_clear_bit(6));
        assert_eq!(Some(3), b.previous_clear_bit(4));
        assert_eq!(Some(0), b.previous_clear_bit(3));
        assert_eq!(Some(0), b.previous_clear_bit(2));
        assert_eq!(Some(0), b.previous_clear_bit(1));
        assert_eq!(None, b.previous_clear_bit(0));

        b.insert(0);
        assert_eq!(None, b.previous_clear_bit(1));
        assert_eq!(None, b.previous_clear_bit(0));
    }

    #[test]
    fn bounds() {
        let mut b: BitSet = BitSet::new();
        assert_eq!(None, b.next_set_bit(usize::max_value()));


        b.insert(0);
        assert_eq!(Some(0), b.previous_set_bit(1));
        assert_eq!(None, b.previous_set_bit(0));
    }

    #[test]
    #[should_panic]
    fn out_of_memory() {
        let mut b: BitSet = BitSet::new();
        let k = usize::max_value() - 1;
        b.insert(k);
        assert_eq!(None, b.next_set_bit(usize::max_value() - 1));
        assert_eq!(None, b.next_set_bit(usize::max_value()));
    }

    quickcheck! {
        fn random_contains(xs: Vec<usize>) -> bool {
            let mut b: BitSet = BitSet::new();

            for i in &xs {
                b.insert(*i);
                if !b.contains(*i) {
                    return false
                }
            }

            true
        }

        fn random_items(xs: Vec<usize>) -> bool {
            let mut b: BitSet = BitSet::new();
            let mut h: HashSet<usize> = HashSet::new();
            for i in &xs {
                b.insert(*i);
                h.insert(*i);
                if h.len() != b.len() {
                    return false
                }
            }

            for i in &h {
                if !b.contains(*i) {
                    return false
                }
            }

            for i in b {
                if !h.contains(&i) {
                    return false
                }
            }

            return true
        }
    }
}