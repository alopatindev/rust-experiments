pub fn binary_search<T>(a: &[T], pattern: &T) -> Option<usize>
    where T: PartialOrd
{
    let n = a.len();
    if n == 0 {
        return None;
    }

    let last = n - 1;

    let mut low = 0;
    let mut high = last;

    while low <= high {
        // let mid = (low + high) / 2; // buggy version
        let mid = low + (high - low) / 2;
        if a[mid] == *pattern {
            return Some(mid);
        }
        if a[mid] > *pattern && mid > 0 {
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
    extern crate rand;

    use rand::Rng;
    use super::*;
    use test::Bencher;

    const BENCH_MAX_N: usize = 1000;

    #[test]
    fn simple() {
        let a: Vec<i32> = vec![10, 22, 55, 66, 66, 333, 1234, 6689];

        assert_eq!(Some(2), binary_search(&a, &55));
        assert_eq!(Some(5), binary_search(&a, &333));

        {
            let result = binary_search(&a, &66);
            assert!(Some(3) == result || Some(4) == result);
        }

        assert_eq!(Some(0), binary_search(&a, &10));
        assert_eq!(Some(1), binary_search(&a, &22));
        assert_eq!(Some(7), binary_search(&a, &6689));
        assert_eq!(Some(6), binary_search(&a, &1234));

        assert_eq!(None, binary_search(&a, &8));
        assert_eq!(None, binary_search(&a, &11));
        assert_eq!(None, binary_search(&a, &1300));
        assert_eq!(None, binary_search(&a, &10000));
    }

    #[test]
    fn empty() {
        let a: Vec<i32> = vec![];
        assert_eq!(None, binary_search(&a, &1));
    }

    /// works on `x86_64` Linux after `echo -n 1 > /proc/sys/vm/overcommit_memory`
    // #[test]
    // #[ignore]
    // #[allow(transmute_ptr_to_ref)]
    // fn heavy() {
    //     extern crate libc;
    //     use std::{mem, ptr};
    //
    //     const N: usize = ((1usize << 46) + (1usize << 44));
    //
    //     type T = i8;
    //     let tsize: usize = mem::size_of::<T>();
    //
    //     let last_array_index = N - 1;
    //     let last_byte_offset: usize = (N - 1) * tsize;
    //     let last_item: T = 120;
    //
    //     let offset0: usize = last_byte_offset / 2;
    //     let offset1: isize = offset0 as isize;
    //     let offset2: isize = (last_byte_offset - offset0) as isize;
    //
    //     unsafe {
    //         println!("trying to allocate {} bytes (~{} GiB) for {} items",
    //                  tsize * N,
    //                  (tsize * N) as f64 / 1073741824.0,
    //                  N);
    //
    //         let p: *mut libc::c_void = libc::calloc(tsize, N);
    //         mem::forget(p);
    //
    //         if p.is_null() {
    //             panic!("cannot allocate memory");
    //         }
    //
    //         println!("adding to vec");
    //         let p_to_last = p.offset(offset1).offset(offset2);
    //         ptr::write(p_to_last as *mut T, last_item);
    //
    //         let a: &[T; N] = mem::transmute(p);
    //         assert_eq!(N, a.len());
    //
    //         assert_eq!(Some(last_array_index), binary_search(a, &last_item));
    //         assert_eq!(None, binary_search(a, &(-10)));
    //         assert_eq!(None, binary_search(a, &12));
    //         assert_eq!(None, binary_search(a, &80));
    //         assert_eq!(None, binary_search(a, &90));
    //         assert_eq!(None, binary_search(a, &100));
    //         assert_eq!(None, binary_search(a, &110));
    //         assert_eq!(None, binary_search(a, &126));
    //         assert_eq!(None, binary_search(a, &127));
    //
    //         libc::free(p);
    //     }
    //
    //     println!("success!");
    // }

    fn make_random_sorted_vec(n: usize) -> Vec<i32> {
        let mut rng = rand::thread_rng();
        let mut result = Vec::with_capacity(n);

        for _ in 0..n {
            result.push(rng.gen());
        }

        result.sort();
        result
    }

    #[bench]
    fn bench_std(b: &mut Bencher) {
        b.iter(|| {
            for n in 0..BENCH_MAX_N {
                let a = make_random_sorted_vec(n);
                let _ = a.binary_search(&22);
            }
        })
    }

    #[bench]
    fn bench_simple(b: &mut Bencher) {
        b.iter(|| {
            for n in 0..BENCH_MAX_N {
                let a = make_random_sorted_vec(n);
                let _ = binary_search(&a[..], &22);
            }
        })
    }
}
