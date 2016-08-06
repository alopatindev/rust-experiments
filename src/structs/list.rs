pub struct Node<'a> {
    data: i32,
    next: Option<&'a Node<'a>>,
}

pub struct NodeIterator<'a> {
    current: Option<&'a Node<'a>>,
}

impl<'a> Iterator for NodeIterator<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        let result = match self.current {
            Some(node) => {
                self.current = node.next;
                Some(node.data)
            }
            None => return None,
        };
        result
    }
}

impl<'a> Node<'a> {
    pub fn new(data: i32, next: Option<&'a Node<'a>>) -> Self {
        Node {
            data: data,
            next: next,
        }
    }

    pub fn push(&'a self, data: i32) -> Node<'a> {
        Node::new(data, Some(&self))
    }

    pub fn iter(&'a self) -> NodeIterator<'a> {
        NodeIterator { current: Some(self) }
    }

    pub fn print(&'a self) {
        for i in self.iter() {
            print!("{} ", i);
        }
        println!("");
    }

    pub fn sum(&self) -> i32 {
        self.iter().fold(0, |a, b| a + b)
    }
}


#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn simple() {
        // let xs = LNode(1, LNode(2, LNode(3)));
        // let ys = LNode(3, LNode(2, LNode(1)));
        // assert!(xs != ys);
        // assert_eq!(xs, xs);
        // assert_eq!(reverse_list(xs), ys);
    }
}
