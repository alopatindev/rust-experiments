#![feature(libc)]
extern crate libc;
use std::mem;
use std::ptr;

fn binary_search<T: std::cmp::PartialOrd>(a: &[T], pattern: &T) -> Option<usize> {
    let last: usize = a.len() - 1;

    let mut low: usize = 0;
    let mut high: usize = last;

    while low <= high {
        let mid = (low + high) / 2;             // buggy version
        //let mid = low + (high - low) / 2;
        //println!("low={} high={} mid={} max={}", low, high, mid, usize::max_value());
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

/// works on x86_64 Linux after `echo -n 1 > /proc/sys/vm/overcommit_memory`
fn test_binary_search_heavy() {
    //const N: usize = (1usize << 45) - 1;
    const N: usize = 1usize << 44;

    type T = i32;
    let tsize: usize = mem::size_of::<T>();

    let last_array_index = N - 1;
    let last_byte_offset: usize = (N - 1) * tsize;
    let last_item: T = 10;

    let offset0: usize = last_byte_offset / 2;
    let offset1: isize = offset0 as isize;
    let offset2: isize = (last_byte_offset - offset0) as isize;

    unsafe {
        println!(
            "trying to allocate {} bytes ({} GiB) for {} items",
            tsize * N,
            (tsize * N) >> 30,
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
        
        libc::free(p);
    }
}

fn main() {
    test_binary_search_heavy();
}
