pub struct List<T> {
    head: Link<T>,
    size: usize,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    data: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            size: 0,
        }
    }

    pub fn push(&mut self, data: T) {
        let new_node = Box::new(Node {
            data: data,
            next: self.head.take(),
        });

        self.head = Some(new_node);
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|boxed_node| {
            let node = *boxed_node;
            self.head = node.next;
            self.size -= 1;
            node.data
        })
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|boxed_node| &boxed_node.data)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|boxed_node| &mut boxed_node.data)
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
        let mut link = self.head.take();
        while let Some(mut boxed_node) = link {
            link = boxed_node.next.take();
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

        assert_eq!(xs.peek(), None);
        assert_eq!(xs.pop(), None);

        let vec = vec![1, 2, 3];
        for &i in &vec {
            xs.push(i);
        }

        assert_eq!(xs.peek(), Some(&3));

        assert_eq!(xs.pop(), Some(3));
        assert_eq!(xs.pop(), Some(2));

        xs.push(4);
        xs.push(5);

        assert_eq!(xs.peek(), Some(&5));
        assert_eq!(xs.peek_mut(), Some(&mut 5));

        xs.peek_mut().map(|mut data| {
            *data = 55;
        });
        assert_eq!(xs.peek(), Some(&55));
        assert_eq!(xs.pop(), Some(55));

        assert_eq!(xs.pop(), Some(4));
        assert_eq!(xs.pop(), Some(1));
        assert_eq!(xs.pop(), None);

        assert_eq!(xs.peek(), None);
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
