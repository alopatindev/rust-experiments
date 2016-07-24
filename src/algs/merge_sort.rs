use std;

fn split(low: usize, high: usize) -> (usize, usize, usize, usize) {
    assert!(low <= high);
    let n = high - low + 1;
    let half = n / 2 - 1;
    (low, low + half, low + half + 1, high)
}

fn merge<T: std::cmp::PartialOrd + std::fmt::Display + std::fmt::Debug + Copy>(a: &Vec<T>, low: usize, high: usize) -> Vec<T> {
    //println!("merge a={:?} low={} high={}", *a, low, high);
    assert!(low <= high);

    let mut result: Vec<T> = vec![];

    let (low1, high1, low2, high2) = split(low, high);

    let mut j = low1;
    let mut k = low2;

    while j <= high1 && k <= high2 {
        if a[j] < a[k] {
            result.push(a[j]);
            j += 1;
        } else {
            result.push(a[k]);
            k += 1;
        }
    }

    while j <= high1 {
        result.push(a[j]);
        j += 1;
    }

    while k <= high2 {
        result.push(a[k]);
        k += 1;
    }

    result
}

fn helper<T: std::cmp::PartialOrd + std::fmt::Display + std::fmt::Debug + Copy>(a: &mut Vec<T>, low: usize, high: usize) -> &mut Vec<T> {
    //println!("helper {:?} {} {}", *a, low, high);
    if low != high {
        let (low1, high1, low2, high2) = split(low, high);
        helper(a, low1, high1);
        helper(a, low2, high2);
        let merged = merge(a, low, high);
        for i in 0..merged.len() {
            a[low + i] = merged[i];
        }
    }
    a
}

pub fn merge_sort<T: std::cmp::PartialOrd + std::fmt::Display + std::fmt::Debug + Copy>(a: &mut Vec<T>) -> &mut Vec<T> {
    let n = a.len();
    helper(a, 0, n - 1)
}

#[test]
fn test_merge1() {
    let mut a = vec![6,7,1,2];
    let b = vec![1,2,6,7];
    let n = a.len();
    assert_eq!(b, merge(&mut a, 0, n - 1));
}

#[test]
fn test_merge2() {
    let mut a = vec![3,1,2];
    let b = vec![1,2,3];
    let n = a.len();
    assert_eq!(b, merge(&mut a, 0, n - 1));
}

#[test]
fn test_merge_sort() {
    let mut a: Vec<i32> = vec![4,2,8,9,3,1,0,5,6,7];
    let b: Vec<i32> = (0..10).collect();
    assert_eq!(a.len(), b.len());
    let a = merge_sort(&mut a);
    assert_eq!(b, *a);
}
