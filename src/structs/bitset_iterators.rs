impl IntoIterator for BitSet {
    type Item = usize;
    type IntoIter = BitSetIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitSetIntoIterator {
            set: self,
            index: 0,
        }
    }
}

pub struct BitSetIntoIterator {
    set: BitSet,
    index: usize,
}

impl Iterator for BitSetIntoIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.set.next_set_bit(self.index);
        if let Some(index) = next {
            self.index = index;
        }
        next
    }
}

pub struct BitSetIterator<'a> {
    set: &'a BitSet,
    index: usize,
}

impl<'a> Iterator for BitSetIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.set.next_set_bit(self.index);
        if let Some(index) = next {
            self.index = index;
        }
        next
    }
}

impl<'a> DoubleEndedIterator for BitSetIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let previous = self.set.previous_set_bit(self.index);
        if let Some(index) = previous {
            self.index = index;
        }
        previous
    }
}
