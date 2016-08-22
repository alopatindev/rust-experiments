pub struct Iter<'a, T: 'a> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|rc_node| {
            self.next = rc_node.next.as_ref().map(|node| &**node);
            &rc_node.data
        })
    }
}
