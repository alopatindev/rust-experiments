use std::mem;

#[derive(Debug)]
pub struct BitSet {
    buckets: Vec<usize>,  // 0th is the lowest
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
        let mut index = from_index + 1;

        while index < self.max_index() {
            let (bucket_index, bit_index) = self.split_index(index);
            let rest_of_bits = self.buckets[bucket_index] >> bit_index;
            let found = (rest_of_bits & 1) == 1;
            if found {
                return Some(index);
            } else {
                index += 1;
            }
        }

        None
    }

    pub fn next_clear_bit(&self, from_index: usize) -> Option<usize> {
        None
    }

    pub fn set(&mut self, index: usize) {
        if !self.get(index) {
            self.maybe_grow_buckets(index);
            let (bucket_index, bit_index) = self.split_index(index);
            let bit = 1usize << bit_index;
            self.buckets[bucket_index] |= bit;
            self.size += 1;
        }
    }

    pub fn clear(&mut self, index: usize) {
        if self.get(index) {
            let (bucket_index, bit_index) = self.split_index(index);
            let mask = !(1usize << bit_index);
            self.buckets[bucket_index] &= mask;
            self.size -= 1;
        }
    }

    pub fn get(&self, index: usize) -> bool {
        if index > self.max_index() {
            false
        } else {
            let (bucket_index, bit_index) = self.split_index(index);
            let bit = 1usize << bit_index;
            (self.buckets[bucket_index] & bit) > 0
        }
    }

    fn bucket_size_in_bits(&self) -> usize {
        let bucket_size = mem::size_of::<usize>();
        bucket_size * 8
    }

    fn max_index(&self) -> usize {
        (self.buckets.len() * self.bucket_size_in_bits()) - 1
    }

    fn maybe_grow_buckets(&mut self, index: usize) {
        while index > self.max_index() {
            self.buckets.push(0);
        }
    }

    fn split_index(&self, index: usize) -> (usize, usize) {
        let n = self.max_index() + 1;
        let bucket_index = index / self.bucket_size_in_bits();
        let bit_index = (index % n) % self.bucket_size_in_bits();
        (bucket_index, bit_index)
    }
}

// https://stackoverflow.com/questions/30218886/how-to-implement-iterator-and-intoiterator-for-a-simple-struct/30220832#30220832

impl IntoIterator for BitSet {
    type Item = usize;
    type IntoIter = BitSetIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitSetIntoIterator { set: self, index: 0 }
    }
}

pub struct BitSetIntoIterator {
    set: BitSet,
    index: usize,
}

impl Iterator for BitSetIntoIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let next = self.set.next_set_bit(self.index);
        if let Some(index) = next {
            self.index = index;
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    extern crate rand;

    const N: usize = 2048;

    #[test]
    fn test_empty() {
        let b: BitSet = BitSet::new();
        let is_empty_set: bool = (0..N)
            .filter(|i: &usize| { b.get(*i) })
            .count() == 0;
        assert!(is_empty_set);
    }

    #[test]
    fn test_single_item() {
        for i in 0..N {
            let mut b: BitSet = BitSet::new();
            b.set(i);
            let xs: Vec<usize> = (0..N)
                .filter(|j: &usize| b.get(*j))
                .collect();
            let contains_single_item_only = xs.len() == 1 && xs[0] == i;
            assert!(contains_single_item_only);
        }
    }

    #[test]
    fn test_random_items() {
        let mut b: BitSet = BitSet::new();
        let mut h: HashSet<usize> = HashSet::new();
        let mut i = 0;
        while i < N {
            b.set(i);
            h.insert(i);
            i += 1 + (rand::random::<usize>() % N);
            assert_eq!(h.len(), b.len());
        }

        for i in &h {
            assert_eq!(true, b.get(*i));
        }

        for ref i in b.into_iter() {
            assert!(h.contains(i));
        }
    }

    #[test]
    fn test_into_iter() {
        let a = vec![5,55,63,64,65,70,88];
        let mut b: BitSet = BitSet::new();
        for i in &a {
            b.set(*i);
        }

        let mut a_iter = a.iter();
        for ref i in b.into_iter() {
            let j = a_iter.next().unwrap();
            assert_eq!(j, i);
        }
    }

    #[test]
    fn test_set_twice() {
        let mut b: BitSet = BitSet::new();
        for i in 0..N {
            b.set(i);
            assert_eq!(true, b.get(i));
            b.set(i);
            assert_eq!(true, b.get(i));
        }
        assert_eq!(N, b.len());
    }

    #[test]
    fn test_set_and_clear() {
        let mut b: BitSet = BitSet::new();
        for i in 0..N {
            b.set(i);
            assert_eq!(true, b.get(i));
            b.clear(i);
            assert_eq!(false, b.get(i));
        }
        assert_eq!(0, b.len());

        for i in 0..N {
            b.clear(i);
            assert_eq!(false, b.get(i));
        }
        assert_eq!(0, b.len());
    }

    #[test]
    fn test_next_set_bit() {
        let a = vec![5,55,63,64,65,70,88];
        let mut b: BitSet = BitSet::new();
        for i in &a {
            b.set(*i);
        }

        let mut index = 0;
        for i in &a {
            assert_eq!(*i, b.next_set_bit(index).unwrap());
            index = *i;
        }

        assert_eq!(None, b.next_set_bit(index));
    }
}
