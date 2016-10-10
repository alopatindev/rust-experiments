// aka circular buffer
pub struct RingBuffer<T: PartialEq + Clone + Default> {
    length: usize,
    data: Vec<T>,
    end: usize,
}

impl<T: PartialEq + Clone + Default> RingBuffer<T> {
    pub fn new(length: usize) -> Self {
        RingBuffer {
            length: length,
            data: Vec::with_capacity(length),
            end: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.data.len() < self.length && self.end >= self.data.len() {
            self.data.push(value);
        } else {
            self.data[self.end] = value;
        }

        self.end += 1;
        self.end %= self.length;
    }

    pub fn resize(&mut self, length: usize) {
        if length == self.length {
            return;
        }

        if length < self.length {
            self.data.resize(length, T::default());
        } else {
            self.data.reserve(length);
        }

        self.length = length;
        self.end %= self.length;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut a = RingBuffer::new(5);

        for i in 0..8 {
            a.push(i);
        }
        assert_eq!(vec![5, 6, 7, 3, 4], a.data);

        a.resize(3);
        assert_eq!(vec![5, 6, 7], a.data);

        a.push(8);
        assert_eq!(vec![8, 6, 7], a.data);

        a.push(9);
        assert_eq!(vec![8, 9, 7], a.data);

        a.push(10);
        a.push(11);
        assert_eq!(vec![11, 9, 10], a.data);

        a.push(12);
        assert_eq!(vec![11, 12, 10], a.data);

        a.resize(4);
        a.push(-13);
        assert_eq!(vec![11, 12, -13], a.data);

        a.resize(15);
        assert_eq!(vec![11, 12, -13], a.data);
        assert_eq!(15, a.length);

        for i in 14..26 {
            a.push(i);
        }
        assert_eq!(vec![11, 12, -13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25],
                   a.data);

        for i in 26..29 {
            a.push(i);
        }
        assert_eq!(vec![26, 27, 28, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25],
                   a.data);

        for i in 29..40 {
            a.push(i);
        }
        a.resize(16);
        a.push(40);
        assert_eq!(vec![26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40],
                   a.data);

        a.push(41);
        assert_eq!(vec![26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41],
                   a.data);
    }
}
