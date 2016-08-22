pub struct List<T> {
    head: Link<T>,
    size: usize,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    data: T,
    next: Link<T>,
}

include!("list_mut_iterators.rs");

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

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            next: self.head
                      .as_ref()
                      .map(|boxed_node| &**boxed_node),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            next: self.head
                      .as_mut()
                      .map(|boxed_node| &mut **boxed_node),
        }
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

        assert_eq!(None, xs.peek());
        assert_eq!(None, xs.pop());

        let vec = vec![1, 2, 3];
        for &i in &vec {
            xs.push(i);
        }

        assert_eq!(Some(&3), xs.peek());

        assert_eq!(Some(3), xs.pop());
        assert_eq!(Some(2), xs.pop());

        xs.push(4);
        xs.push(5);

        assert_eq!(Some(&5), xs.peek());
        assert_eq!(Some(&mut 5), xs.peek_mut());

        xs.peek_mut().map(|mut data| {
            *data = 55;
        });
        assert_eq!(Some(&55), xs.peek());
        assert_eq!(Some(55), xs.pop());

        assert_eq!(Some(4), xs.pop());
        assert_eq!(Some(1), xs.pop());
        assert_eq!(None, xs.pop());

        assert_eq!(None, xs.peek());
    }

    #[test]
    fn into_iter() {
        let mut xs = List::new();

        let vec = vec![1, 2, 3];
        for &i in &vec {
            xs.push(i);
        }

        let mut iter = xs.into_iter();
        assert_eq!(Some(3), iter.next());
        assert_eq!(Some(2), iter.next());
        assert_eq!(Some(1), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn iter() {
        let mut xs = List::new();

        let vec = vec![1, 2, 3];
        for &i in &vec {
            xs.push(i);
        }

        let mut iter = xs.iter();
        assert_eq!(Some(&3), iter.next());
        assert_eq!(Some(&2), iter.next());
        assert_eq!(Some(&1), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn iter_mut() {
        let mut xs = List::new();

        let vec = vec![1, 2, 3];
        for &i in &vec {
            xs.push(i);
        }

        {
            let mut iter = xs.iter_mut();
            assert_eq!(Some(&mut 3), iter.next());

            match iter.next() {
                Some(mut x) => {
                    assert_eq!(&2, x);
                    *x = 55;
                }
                None => unreachable!(),
            }

            assert_eq!(Some(&mut 1), iter.next());
            assert_eq!(None, iter.next());
        }

        {
            let mut iter = xs.iter_mut();
            let _ = iter.next();
            assert_eq!(Some(&mut 55), iter.next());
        }
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

        assert_eq!(Some(Hello::new(2)), xs.pop());
        assert_eq!(Some(Hello::new(1)), xs.pop());
        assert_eq!(None, xs.pop());
    }
}
