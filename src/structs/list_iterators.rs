pub struct Iter<'a, T: 'a + Clone + PartialEq> {
    next: Option<&'a Node<T>>,
}

impl<'a, T: Clone + PartialEq> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|rc_node| {
            self.next = rc_node.next.as_ref().map(|node| &**node);
            &rc_node.data
        })
    }
}

impl<T: Clone + PartialEq> List<T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            next: self.head
                      .as_ref()
                      .map(|rc_node| &**rc_node),
        }
    }
}
