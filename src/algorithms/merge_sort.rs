use std::fmt::{Display, Debug};
use std::ops::Range;

pub type Bounds = Range<usize>;

fn split(bounds: &Bounds) -> (Bounds, Bounds) {
    assert!(bounds.start < bounds.end);

    let n = bounds.len();
    let half = n / 2 - 1;

    let left = bounds.start .. bounds.start + half + 1;
    let right = left.end .. bounds.end;

    (left, right)
}

pub fn merge<T: PartialOrd + Display + Debug + Copy>(a: &Vec<T>, bounds: &Bounds) -> Vec<T> {
    assert!(bounds.start < bounds.end);

    let accumulate_result = |index: &mut usize, result: &mut Vec<T>| {
        result.push(a[*index]);
        *index += 1;
    };

    let mut result: Vec<T> = vec![];
    result.reserve_exact(bounds.len());

    let (left, right) = split(&bounds);
    let mut i = left.start;
    let mut j = right.start;

    while i < left.end && j < right.end {
        if a[i] < a[j] {
            accumulate_result(&mut i, &mut result);
        } else {
            accumulate_result(&mut j, &mut result);
        }
    }

    while i < left.end {
        accumulate_result(&mut i, &mut result);
    }

    while j < right.end {
        accumulate_result(&mut j, &mut result);
    }

    result
}

fn merge_and_update<T: PartialOrd + Display + Debug + Copy>(a: &mut Vec<T>, bounds: &Bounds) {
    let merged = merge(a, &bounds);
    for i in 0..merged.len() {
        a[bounds.start + i] = merged[i];
    }
}

pub fn merge_sort_recursive<T: PartialOrd + Display + Debug + Copy>(a: &mut Vec<T>) -> &mut Vec<T> {
    fn helper<T: PartialOrd + Display + Debug + Copy>(a: &mut Vec<T>, bounds: &Bounds) {
        assert!(bounds.start < bounds.end);

        if bounds.len() > 1 {
            let (left, right) = split(&bounds);
            helper(a, &left);
            helper(a, &right);
            merge_and_update(a, &bounds);
        }
    }

    let n = a.len();
    if n > 1 {
        helper(a, &(0..n));
    }

    a
}

pub fn merge_sort_iterative<T: PartialOrd + Display + Debug + Copy>(a: &mut Vec<T>) -> &mut Vec<T> {
    let n = a.len();
    if n <= 1 {
        return a;
    }

    let mut queue: Vec<Bounds> = vec![];
    let mut stack: Vec<Bounds> = vec![];

    queue.push((0..n));
    while let Some(bounds) = queue.pop() {
        if bounds.len() > 1 {
            let (left, right) = split(&bounds);
            stack.push(bounds);
            queue.push(left);
            queue.push(right);
        }
    }

    while let Some(bounds) = stack.pop() {
        if bounds.len() > 1 {
            merge_and_update(a, &bounds);
        }
    }

    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use test;
    extern crate rand;

    #[test]
    fn test_merge_sort() {
        type T = i32;

        fn test_merge_sort_impl(func: fn(&mut Vec<T>) -> &mut Vec<T>) {
            let merge_sort = || {
                let mut a: Vec<i32> = vec![4,2,8,9,3,1,0,5,6,7];
                let b: Vec<i32> = (0..10).collect();
                assert_eq!(a.len(), b.len());
                let a = func(&mut a);
                assert_eq!(b, *a);
            };

            let merge_sort_empty = || {
                let mut a: Vec<i32> = vec![];
                let a = func(&mut a);
                assert!(a.is_empty());
            };

            let merge_sort_single = || {
                let mut a: Vec<i32> = vec![1];
                let a = func(&mut a);
                assert_eq!(1, a.len());
            };

            merge_sort();
            merge_sort_empty();
            merge_sort_single();
        }

        test_merge_sort_impl(merge_sort_iterative);
        test_merge_sort_impl(merge_sort_recursive);
    }

    #[test]
    fn test_merge_1() {
        let mut a = vec![6,7,1,2];
        let b = vec![1,2,6,7];
        let n = a.len();
        assert_eq!(b, merge(&mut a, &(0..n)));
    }

    #[test]
    fn test_merge_2() {
        let mut a = vec![3,1,2];
        let b = vec![1,2,3];
        let n = a.len();
        assert_eq!(b, merge(&mut a, &(0..n)));
    }

    fn make_random_vec(n: usize) -> Vec<i32> {
        let mut a = vec![0; n];
        for i in 0..n {
            a[i] = rand::random::<i32>() % 100;
        }
        a
    }

    const BENCH_MAXN: usize = 1000;

    #[bench]
    fn bench_merge_sort_recursive(b: &mut test::Bencher) {
        b.iter(|| for n in 0..BENCH_MAXN {
            let mut a = make_random_vec(n);
            let _ = merge_sort_recursive(&mut a);
        })
    }

    #[bench]
    fn bench_merge_sort_iterative(b: &mut test::Bencher) {
        b.iter(|| for n in 0..BENCH_MAXN {
            let mut a = make_random_vec(n);
            let _ = merge_sort_iterative(&mut a);
        })
    }
}
