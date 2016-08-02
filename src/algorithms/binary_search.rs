pub fn binary_search<T: PartialOrd>(a: &[T], pattern: &T) -> Option<usize> {
    let n = a.len();
    if n == 0 {
        return None
    }

    let last = n - 1;

    let mut low = 0;
    let mut high = last;

    while low <= high {
        // let mid = (low + high) / 2; // buggy version
        let mid = low + (high - low) / 2;
        if a[mid] == *pattern {
            return Some(mid);
        } if a[mid] > *pattern && mid > 0 {
            high = mid - 1;
        } else if a[mid] < *pattern && mid < last {
            low = mid + 1;
        } else {
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use test;
    extern crate rand;

    #[test]
    fn test_binary_search() {
        let a: Vec<i32> = vec![10, 22, 55, 66, 66, 333, 1234, 6689];

        assert_eq!(binary_search(&a, &55), Some(2));
        assert_eq!(binary_search(&a, &333), Some(5));

        {
            let result = binary_search(&a, &66);
            assert!(result == Some(3) || result == Some(4));
        }

        assert_eq!(binary_search(&a, &10), Some(0));
        assert_eq!(binary_search(&a, &22), Some(1));
        assert_eq!(binary_search(&a, &6689), Some(7));
        assert_eq!(binary_search(&a, &1234), Some(6));

        assert_eq!(binary_search(&a, &8), None);
        assert_eq!(binary_search(&a, &11), None);
        assert_eq!(binary_search(&a, &1300), None);
        assert_eq!(binary_search(&a, &10000), None);
    }

    #[test]
    fn test_binary_search_empty() {
        let a: Vec<i32> = vec![];
        assert_eq!(binary_search(&a, &1), None);
    }

    /// works on x86_64 Linux after `echo -n 1 > /proc/sys/vm/overcommit_memory`
    #[test]
    #[ignore]
    fn test_binary_search_heavy() {
        extern crate libc;
        use std::{mem, ptr};

        const N: usize = ((1usize << 46) + (1usize << 44));

        type T = i8;
        let tsize: usize = mem::size_of::<T>();

        let last_array_index = N - 1;
        let last_byte_offset: usize = (N - 1) * tsize;
        let last_item: T = 120;

        let offset0: usize = last_byte_offset / 2;
        let offset1: isize = offset0 as isize;
        let offset2: isize = (last_byte_offset - offset0) as isize;

        unsafe {
            println!(
                "trying to allocate {} bytes (~{} GiB) for {} items",
                tsize * N,
                (tsize * N) as f64 / 1073741824.0,
                N
                );

            let p = libc::calloc(tsize, N);
            mem::forget(p);

            if p.is_null() {
                panic!("cannot allocate memory");
            }

            println!("adding to vec");
            let p_to_last = p.offset(offset1).offset(offset2);
            ptr::write(p_to_last as *mut T, last_item);

            let a: &[T; N] = mem::transmute(p);
            assert_eq!(a.len(), N);

            assert_eq!(binary_search(a, &last_item), Some(last_array_index));
            assert_eq!(binary_search(a, &(-10)), None);
            assert_eq!(binary_search(a, &12), None);
            assert_eq!(binary_search(a, &80), None);
            assert_eq!(binary_search(a, &90), None);
            assert_eq!(binary_search(a, &100), None);
            assert_eq!(binary_search(a, &110), None);
            assert_eq!(binary_search(a, &126), None);
            assert_eq!(binary_search(a, &127), None);

            libc::free(p);
        }

        println!("success!");
    }

    fn make_random_sorted_vec(n: usize) -> Vec<i32> {
        let mut a = vec![0; n];
        let mut k = 0;
        for i in 0..n {
            k += rand::random::<i32>() % 100;
            a[i] = k;
        }
        a
    }

    const BENCH_MAX_N: usize = 1000;

    #[bench]
    fn bench_binary_search_std(b: &mut test::Bencher) {
        b.iter(|| for n in 0..BENCH_MAX_N {
            let a = make_random_sorted_vec(n);
            let _ = a.binary_search(&22);
        })
    }

    #[bench]
    fn bench_binary_search(b: &mut test::Bencher) {
        b.iter(|| for n in 0..BENCH_MAX_N {
            let a = make_random_sorted_vec(n);
            let _ = binary_search(&a[..], &22);
        })
    }
}
