use std::fmt::*;
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

fn helper<T: PartialOrd + Display + Debug + Copy>(a: &mut Vec<T>, bounds: &Bounds) {
    assert!(bounds.start < bounds.end);

    if bounds.len() > 1 {
        let (left, right) = split(&bounds);
        helper(a, &left);
        helper(a, &right);

        let merged = merge(a, &bounds);
        for i in 0..merged.len() {
            a[bounds.start + i] = merged[i];
        }
    }
}

pub fn merge_sort<T: PartialOrd + Display + Debug + Copy>(a: &mut Vec<T>) -> &mut Vec<T> {
    let n = a.len();

    if n > 0 {
        helper(a, &(0..n));
    }

    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use test;

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

    #[test]
    fn test_merge_sort() {
        let mut a: Vec<i32> = vec![4,2,8,9,3,1,0,5,6,7];
        let b: Vec<i32> = (0..10).collect();
        assert_eq!(a.len(), b.len());
        let a = merge_sort(&mut a);
        assert_eq!(b, *a);
    }

    #[test]
    fn test_merge_sort_empty() {
        let mut a: Vec<i32> = vec![];
        let a = merge_sort(&mut a);
        assert_eq!(0, a.len());
    }

    #[test]
    fn test_merge_sort_single() {
        let mut a: Vec<i32> = vec![1];
        let a = merge_sort(&mut a);
        assert_eq!(1, a.len());
    }

    #[bench]
    #[ignore]
    fn bench_merge_sort(b: &mut test::Bencher) {
        b.iter(|| {
            let mut a = vec![1,4,0,45,6];
            merge_sort(&mut a);
        })
    }
}
