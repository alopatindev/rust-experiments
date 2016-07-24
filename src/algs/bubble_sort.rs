use std::fmt::*;

pub fn bubble_sort<T: PartialOrd + Display + Debug>(a: &mut Vec<T>) -> &mut Vec<T> {
    let mut n = a.len();
    if n == 0 {
        return a
    }

    let mut swapped = true;
    while swapped {
        let mut i = 0;
        swapped = false;
        while i < n - 1 {
            if a[i] > a[i + 1] {
                a.swap(i, i + 1);
                swapped = true;
            }
            i += 1;
        }
        n -= 1;
    }

    a
}

#[test]
fn test_bubble_sort() {
    let mut a: Vec<i32> = vec![4,2,8,9,3,1,0,5,6,7];
    let b: Vec<i32> = (0..10).collect();
    assert_eq!(a.len(), b.len());
    let a = bubble_sort(&mut a);
    assert!(*a == b);
}

#[test]
fn test_bubble_sort_empty() {
    let mut a: Vec<i32> = vec![];
    let a = bubble_sort(&mut a);
    assert_eq!(0, a.len());
}

#[test]
fn test_bubble_sort_single() {
    let mut a: Vec<i32> = vec![1];
    let a = bubble_sort(&mut a);
    assert_eq!(1, a.len());
}
