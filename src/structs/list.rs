use std::mem;

pub struct List {
    head: Link,
    size: usize,
}

enum Link {
    Empty,
    More(Box<Node>),
}

struct Node {
    elem: i32,
    next: Link,
}

impl List {
    pub fn new() -> Self {
        List {
            head: Link::Empty,
            size: 0,
        }
    }

    pub fn push(&mut self, elem: i32) {
        let new_node = Box::new(Node {
            elem: elem,
            next: mem::replace(&mut self.head, Link::Empty),
        });

        self.head = Link::More(new_node);
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<i32> {
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

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for List {
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
        let mut xs = List::new();

        assert_eq!(0, xs.len());
        assert!(xs.is_empty());
        assert_eq!(xs.pop(), None);

        let vec = vec![1, 2, 3];
        for i in &vec {
            xs.push(*i);
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
}
