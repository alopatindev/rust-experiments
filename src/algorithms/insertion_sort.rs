use std::fmt::{Debug, Display};

pub fn insertion_sort<T>(a: &mut Vec<T>) -> &mut Vec<T>
    where T: PartialOrd + Display + Debug
{
    let n = a.len();
    if n == 0 {
        return a;
    }

    let mut i = 0;
    while i < n - 1 {
        let mut j = i + 1;
        while j > 0 && a[j] < a[j - 1] {
            a.swap(j, j - 1);
            j -= 1;
        }
        i += 1;
    }

    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut a: Vec<i32> = vec![4, 2, 8, 9, 3, 1, 0, 5, 6, 7];
        let b: Vec<i32> = (0..10).collect();
        assert_eq!(a.len(), b.len());
        let a = insertion_sort(&mut a);
        assert!(*a == b);
    }

    #[test]
    fn empty() {
        let mut a: Vec<i32> = vec![];
        let a = insertion_sort(&mut a);
        assert_eq!(0, a.len());
    }

    #[test]
    fn single() {
        let mut a: Vec<i32> = vec![1];
        let a = insertion_sort(&mut a);
        assert_eq!(1, a.len());
    }
}
