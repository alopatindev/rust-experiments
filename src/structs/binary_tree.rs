use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct BinaryTree<T: Clone + PartialEq> {
    root: Link<T>,
}

pub type Link<T> = Option<Rc<Node<T>>>;

#[derive(Debug)]
pub struct Node<T: Clone + PartialEq> {
    data: T,
    left: Link<T>,
    right: Link<T>,
}

impl<T: Clone + PartialEq> BinaryTree<T> {
    pub fn new(data: T, left: &BinaryTree<T>, right: &BinaryTree<T>) -> Self {
        let rc_node = Rc::new(Node {
            data: data,
            left: left.root.clone(),
            right: right.root.clone(),
        });

        BinaryTree { root: Some(rc_node) }
    }

    pub fn new_leaf(data: T) -> Self {
        let rc_node = Rc::new(Node {
            data: data,
            left: None,
            right: None,
        });

        BinaryTree { root: Some(rc_node) }
    }

    pub fn from_node(node: &Link<T>) -> BinaryTree<T> {
        BinaryTree { root: node.clone() }
    }

    pub fn new_empty() -> Self {
        BinaryTree { root: None }
    }

    pub fn data(&self) -> Option<&T> {
        self.root.as_ref().map(|rc_node| &rc_node.data)
    }

    pub fn left_data(&self) -> Option<&T> {
        self.root.as_ref().map_or(None, |rc_node| {
            rc_node.left.as_ref().map(|rc_left_node| &rc_left_node.data)
        })
    }

    pub fn right_data(&self) -> Option<&T> {
        self.root.as_ref().map_or(None, |rc_node| {
            rc_node.right.as_ref().map(|rc_right_node| &rc_right_node.data)
        })
    }

    pub fn left(&self) -> BinaryTree<T> {
        self.root
            .as_ref()
            .map_or(BinaryTree::new_empty(),
                    |rc_node| BinaryTree::from_node(&rc_node.as_ref().left))
    }

    pub fn right(&self) -> BinaryTree<T> {
        self.root
            .as_ref()
            .map_or(BinaryTree::new_empty(),
                    |rc_node| BinaryTree::from_node(&rc_node.as_ref().right))
    }

    pub fn is_leaf(&self) -> bool {
        match self.root {
            Some(ref r) => r.left.is_none() && r.right.is_none(),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let empty = BinaryTree::new_empty();
        let left = BinaryTree::new_leaf(0);
        let right_right = BinaryTree::new_leaf(3);
        let right = BinaryTree::new(2, &empty, &right_right);
        let root = BinaryTree::new(1, &left, &right);

        assert_eq!(Some(&1), root.data());
        assert_eq!(Some(&0), root.left_data());
        assert_eq!(Some(&2), root.right_data());
        assert_eq!(None, root.right().left_data());
        assert_eq!(Some(&3), root.right().right_data());

        assert!(!root.is_leaf());
        assert!(!right.is_leaf());
        assert!(right_right.is_leaf());
        assert!(left.is_leaf());
        assert!(empty.is_leaf());
    }
}
