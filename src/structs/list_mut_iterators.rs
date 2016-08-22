pub struct IntoIter<T>(List<T>);

pub struct Iter<'a, T: 'a> {
    next: Option<&'a Node<T>>,
}

pub struct IterMut<'a, T: 'a> {
    next: Option<&'a mut Node<T>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|boxed_node| {
            self.next = boxed_node.next.as_ref().map(|node| &**node);
            &boxed_node.data
        })
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|boxed_node| {
            self.next = boxed_node.next.as_mut().map(|node| &mut **node);
            &mut boxed_node.data
        })
    }
}

impl<T> List<T> {
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
