use std::sync::Arc;

pub struct List<T: Clone + PartialEq> {
    head: Link<T>,
}

type Link<T> = Option<Arc<Node<T>>>;

struct Node<T: Clone + PartialEq> {
    data: T,
    next: Link<T>,
    size: usize,
}

include!("list_iterators.rs");

impl<T: Clone + PartialEq> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn append(&self, data: T) -> Self {
        let node = Node {
            data: data,
            next: self.head.clone(),
            size: self.len(),
        };

        let head = Some(Arc::new(node));
        List { head: head }
    }

    pub fn tail(&self) -> Self {
        List {
            head: self.head
                .as_ref()
                .and_then(|rc_node| rc_node.next.clone()),
        }
    }

    pub fn head(&self) -> Option<&T> {
        self.head
            .as_ref()
            .map(|rc_node| &rc_node.data)
    }

    pub fn len(&self) -> usize {
        self.head
            .as_ref()
            .map_or_else(|| 0, |rc_node| rc_node.size + 1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, data: T) -> bool {
        self.iter()
            .filter(|&d| *d == data)
            .next()
            .is_some()
    }

    pub fn skip(&self, n: usize) -> Self {
        let mut head = self.head.as_ref();
        let mut i = 0;

        while let Some(rc_node) = head {
            if i < n {
                head = rc_node.next.as_ref();
                i += 1;
            } else {
                return List { head: Some(rc_node.clone()) };
            }
        }

        List::new()
    }

    pub fn take(&self, n: usize) -> Self {
        let mut xs = List::new();
        let mut i = 0;

        for it in self.iter() {
            if i < n {
                xs = xs.append(it.clone());
                i += 1;
            } else {
                break;
            }
        }

        let mut ys = List::new();
        for it in xs.iter() {
            ys = ys.append(it.clone());
        }

        ys
    }
}

impl<T: Clone + PartialEq> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(rc_node) = head {
            if let Ok(mut node) = Arc::try_unwrap(rc_node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::*;

    #[test]
    fn simple() {
        let empty = List::new();
        assert_eq!(None, empty.head());
        assert_eq!(0, empty.len());
        assert!(empty.is_empty());

        let xs = empty.append(1)
            .append(2)
            .append(3);

        assert_eq!(false, xs.is_empty());
        assert_eq!(3, xs.len());
        assert_eq!(2, xs.tail().len());
        assert_eq!(1, xs.tail().tail().len());
        assert_eq!(0, xs.tail().tail().tail().len());

        assert_eq!(Some(&3), xs.head());
        assert_eq!(Some(&2), xs.tail().head());
        assert_eq!(Some(&1), xs.tail().tail().head());
        assert_eq!(None, xs.tail().tail().tail().head());

        assert_eq!(Some(&3), xs.skip(0).head());
        assert_eq!(Some(&2), xs.skip(1).head());
        assert_eq!(Some(&1), xs.skip(1).tail().head());
        assert_eq!(Some(&1), xs.skip(2).head());
        assert_eq!(None, xs.skip(3).head());

        assert_eq!(None, xs.take(0).head());
        assert_eq!(Some(&3), xs.take(1).head());
        assert_eq!(None, xs.take(1).tail().head());
        assert_eq!(Some(&3), xs.take(2).head());
        assert_eq!(Some(&2), xs.take(2).tail().head());
        assert_eq!(None, xs.take(2).tail().tail().head());

        assert!(xs.contains(1));
        assert!(xs.contains(2));
        assert!(xs.contains(3));
        assert!(!xs.contains(4));

        let mut iter = xs.iter();
        assert_eq!(Some(&3), iter.next());
        assert_eq!(Some(&2), iter.next());
        assert_eq!(Some(&1), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn threaded() {
        let mut a_workers = vec![];
        let mut b_workers = vec![];

        for i in 0..10 {
            let xs = List::<i32>::new().append(i);
            let a = thread::spawn(move || xs.append(i + 1));
            a_workers.push(a);
        }

        for w in a_workers {
            let xs = w.join().unwrap();
            let b = thread::spawn(move || xs.append(5));
            b_workers.push(b);
        }

        for w in b_workers {
            let xs = w.join().unwrap();
            assert_eq!(3, xs.len());
            assert!(xs.contains(5));
        }
    }
}
