pub struct MaxHeap<T> {
    keys: Vec<T>,
    size: usize,
}

const ROOT: usize = 0;

impl<T: PartialOrd + PartialEq + Eq + Clone> MaxHeap<T> {
    pub fn new() -> Self {
        MaxHeap {
            keys: vec![],
            size: 0,
        }
    }

    pub fn insert(&mut self, x: T) {
        self.keys.push(x);
        self.size += 1;
        let len = self.size;
        self.sift_up(len - 1);
    }

    pub fn max(&self) -> &T {
        &self.keys[ROOT]
    }

    pub fn build(&mut self, unsorted: &[T]) {
        self.keys = unsorted.to_vec();
        self.size = self.keys.len();

        let first_leaf = self.len() / 2;
        let non_leaves = ROOT..first_leaf;

        for i in non_leaves.rev() {
            self.heapify(i);
        }
    }

    pub fn heapify(&mut self, i: usize) {
        if !self.property_satisfied(i) {
            let child = self.max_child(i);
            self.keys.swap(child, i);
            self.heapify(child);
        }
    }

    pub fn sort(&mut self, unsorted: &[T]) {
        self.build(unsorted);

        while self.len() > 1 {
            let _ = self.extract_max();
        }

        self.size = self.keys.len();
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn parent(&self, i: usize) -> usize {
        i / 2
    }

    fn left(&self, i: usize) -> usize {
        2 * i
    }

    fn right(&self, i: usize) -> usize {
        2 * i + 1
    }

    fn has_left(&self, i: usize) -> bool {
        self.left(i) < self.len()
    }

    fn has_right(&self, i: usize) -> bool {
        self.right(i) < self.len()
    }

    fn max_child(&self, i: usize) -> usize {
        assert!(self.has_left(i));

        let mut result = self.left(i);

        if self.has_right(i) && *self.right_key(i) >= self.keys[result] {
            result = self.right(i);
        }

        result
    }

    fn left_key(&self, i: usize) -> &T {
        assert!(self.has_left(i));
        &self.keys[self.left(i)]
    }

    fn right_key(&self, i: usize) -> &T {
        assert!(self.has_right(i));
        &self.keys[self.right(i)]
    }

    fn property_satisfied(&self, i: usize) -> bool {
        let left = !self.has_left(i) || self.keys[i] >= *self.left_key(i);
        let right = !self.has_right(i) || self.keys[i] >= *self.right_key(i);
        left && right
    }

    fn sift_up(&mut self, i: usize) {
        let mut j = i;
        while j != ROOT {
            let p = self.parent(j);
            if self.property_satisfied(p) {
                break;
            } else {
                self.keys.swap(p, j);
                j = p;
            }
        }
    }

    fn extract_max(&mut self) -> T {
        let result = self.keys[ROOT].clone();
        self.keys.swap(ROOT, self.size - 1);
        self.size -= 1;
        self.heapify(ROOT);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut h: MaxHeap<i32> = MaxHeap::new();
        assert!(h.is_empty());

        h.build(&[1, 8, -12, 4, 25, 2]);

        for i in 0..h.len() {
            assert!(h.property_satisfied(i));
        }

        assert_eq!(25, *h.max());
        assert_eq!(25, h.extract_max());
        assert_eq!(8, *h.max());
    }

    #[test]
    fn insertion() {
        let mut h: MaxHeap<i32> = MaxHeap::new();

        for i in &[1, 8, -12, 4, 25, 2] {
            h.insert(*i as i32);
        }

        for i in 0..h.len() {
            assert!(h.property_satisfied(i));
        }
    }

    #[test]
    fn sorting() {
        let mut h: MaxHeap<i32> = MaxHeap::new();

        let mut unsorted = vec![1, 8, -12, 4, 25, 2, -123, 2];
        h.sort(&unsorted[..]);

        unsorted.sort();
        let sorted = unsorted;

        assert_eq!(sorted, h.keys);
    }
}
