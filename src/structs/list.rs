use std::rc::Rc;

pub struct List<T: Clone> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T: Clone> {
    data: T,
    next: Link<T>,
}

include!("list_iterators.rs");

impl<T: Clone> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn append(&self, data: T) -> Self {
        let node = Node {
            data: data,
            next: self.head.clone(),
        };

        let head = Some(Rc::new(node));
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

impl<T: Clone> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(rc_node) = head {
            if let Ok(mut node) = Rc::try_unwrap(rc_node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let empty = List::new();
        assert_eq!(None, empty.head());

        let xs = empty.append(1)
                      .append(2)
                      .append(3);

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

        let mut iter = xs.iter();
        assert_eq!(Some(&3), iter.next());
        assert_eq!(Some(&2), iter.next());
        assert_eq!(Some(&1), iter.next());
        assert_eq!(None, iter.next());
    }
}
