use std::mem;

pub struct List<T> {
    head: Link<T>,
    size: usize,
}

enum Link<T> {
    Empty,
    More(Box<Node<T>>),
}

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: Link::Empty,
            size: 0,
        }
    }

    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem: elem,
            next: mem::replace(&mut self.head, Link::Empty),
        });

        self.head = Link::More(new_node);
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None,
            Link::More(boxed_node) => {
                let node = *boxed_node;
                self.head = node.next;
                self.size -= 1;
                Some(node.elem)
            }
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut it = mem::replace(&mut self.head, Link::Empty);
        while let Link::More(mut boxed_node) = it {
            it = mem::replace(&mut boxed_node.next, Link::Empty);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut xs = List::<i32>::new();

        assert_eq!(0, xs.len());
        assert!(xs.is_empty());
        assert_eq!(xs.pop(), None);

        let vec = vec![1, 2, 3];
        for &i in &vec {
            xs.push(i);
        }

        assert_eq!(xs.pop(), Some(3));
        assert_eq!(xs.pop(), Some(2));

        xs.push(4);
        xs.push(5);

        assert_eq!(xs.pop(), Some(5));
        assert_eq!(xs.pop(), Some(4));
        assert_eq!(xs.pop(), Some(1));
        assert_eq!(xs.pop(), None);
    }

    #[derive(PartialEq, Debug)]
    struct Hello {
        id: usize,
    }

    impl Hello {
        pub fn new(id: usize) -> Self {
            Hello { id: id }
        }
    }

    #[test]
    fn objects() {
        let mut xs = List::<Hello>::new();
        assert_eq!(xs.pop(), None);

        let vec = vec![1, 2];
        for &i in &vec {
            xs.push(Hello::new(i));
        }

        assert_eq!(xs.pop(), Some(Hello::new(2)));
        assert_eq!(xs.pop(), Some(Hello::new(1)));
        assert_eq!(xs.pop(), None);
    }
}
