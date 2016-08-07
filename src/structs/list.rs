pub type BoxNode<T> = Box<Node<T>>;
pub type OptionBoxNode<T> = Option<BoxNode<T>>;

pub struct Node<T: PartialEq> {
    data: T,
    next: OptionBoxNode<T>,
}

pub struct List<T: PartialEq> {
    head: OptionBoxNode<T>,
    size: usize,
}

impl<T: PartialEq> List<T> {
    pub fn new() -> List<T> {
        List {
            head: None,
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, data: T) {
        self.size += 1;
        let tail = self.head.take();
        let head = Node {
            data: data,
            next: tail,
        };
        self.head = Some(Box::new(head));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut xs: List<i32> = List::new();
        assert_eq!(0, xs.len());
        assert!(xs.is_empty());

        xs.push(1);
        xs.push(2);
        xs.push(3);
        assert!(!xs.is_empty());
        assert_eq!(3, xs.len());

        assert_eq!(3, xs.head.unwrap().data);
    }
}
