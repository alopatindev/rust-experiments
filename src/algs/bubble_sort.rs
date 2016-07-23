use std;

pub fn bubble_sort<T: std::cmp::PartialOrd + std::fmt::Display + std::fmt::Debug>(a: &mut Vec<T>) -> &mut Vec<T> {
    let mut swapped = true;
    while swapped {
        let mut i = 0;
        swapped = false;
        while i < a.len() - 1 {
            if a[i] > a[i + 1] {
                a.swap(i, i + 1);
                swapped = true;
                //println!("{:?} i={}", *a, i);
            }
            i += 1;
        }
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
