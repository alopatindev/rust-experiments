use std::fmt::*;

pub fn insertion_sort<T: PartialOrd + Display + Debug>(a: &mut Vec<T>) -> &mut Vec<T> {
    let mut i = 0;
    while i < a.len() - 1 {
        let mut j = i + 1;
        while j > 0 && a[j] < a[j - 1] {
            a.swap(j, j - 1);
            j -= 1;
        }
        i += 1;
    }
    a
}

#[test]
fn test_insertion_sort() {
    let mut a: Vec<i32> = vec![4,2,8,9,3,1,0,5,6,7];
    let b: Vec<i32> = (0..10).collect();
    assert_eq!(a.len(), b.len());
    let a = insertion_sort(&mut a);
    assert!(*a == b);
}
