pub enum List<T> {
    Nil,
    Cons(T, Box<List<T>>),
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn simple() {}
}
