pub fn quick_sort<T>(a: &mut Vec<T>) -> &mut Vec<T>
    where T: PartialOrd + Copy
{
    let n = a.len();
    if n == 0 {
        return a;
    }

    let mut bounds = Vec::with_capacity(n);
    bounds.push((0, n - 1));

    while let Some((low, high)) = bounds.pop() {
        let pivot = partition(a, low, high);

        if pivot > 0 && low < pivot - 1 {
            bounds.push((low, pivot - 1));
        }

        if pivot < n && high > pivot + 1 {
            bounds.push((pivot + 1, high));
        }
    }

    a
}

#[allow(collapsible_if)]
pub fn partition<T>(a: &mut Vec<T>, mut low: usize, mut high: usize) -> usize
    where T: PartialOrd + Copy
{
    let mut pivot = low;

    while low != high {
        if pivot == low {
            if a[high] < a[pivot] {
                a.swap(high, pivot);
                pivot = high;
            } else {
                high -= 1;
            }
        } else {
            if a[low] > a[pivot] {
                a.swap(low, pivot);
                pivot = low;
            } else {
                low += 1;
            }
        }
    }

    pivot
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use rand::Rng;
    use super::*;
    use test::Bencher;

    const BENCH_MAX_N: usize = 1000;

    quickcheck! {
        fn random_items(a: Vec<i32>) -> bool {
            let mut a = a.clone();
            let mut b = a.clone();
            b.sort();
            b.as_slice() == quick_sort(&mut a).as_slice()
        }
    }

    #[test]
    fn test() {
        // example from https://www.youtube.com/watch?v=3OLTJlwyIqQ
        let mut a = vec![5, 2, 6, 1, 3, 4];
        let mut b = a.clone();
        quick_sort(&mut a);
        b.sort();
        assert_eq!(b.as_slice(), quick_sort(&mut a).as_slice());
    }

    #[bench]
    fn bench(b: &mut Bencher) {
        b.iter(|| {
            for n in 0..BENCH_MAX_N {
                let mut a = make_random_vec(n);
                let _ = quick_sort(&mut a);
            }
        })
    }

    fn make_random_vec(n: usize) -> Vec<i32> {
        let mut rng = rand::thread_rng();
        let mut result = Vec::with_capacity(n);

        for _ in 0..n {
            result.push(rng.gen());
        }

        result
    }
}
