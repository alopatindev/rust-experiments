extern crate rust_experiments;
use rust_experiments::*;

fn array_search() {
    let a: Vec<i32> = vec![10, 22, 55, 66, 66, 333, 1234, 6689];
    assert_eq!(algorithms::binary_search::binary_search(&a, &55), Some(2));
}

fn test_bitset() {
    let a = vec![10, 22, 55, 66, 66, 333, 1234, 6689];
    let mut set: structs::bitset::BitSet = structs::bitset::BitSet::new();
    println!("empty set={}", set);
    for i in &a {
        set.insert(*i);
    }
    println!("set={}", set);

    let unset = a[1];
    set.remove(unset);
    println!("set={} with unset={}", set, unset);
}

fn test_strings() {
    let s = "Nы";
    assert_eq!(3, s.len());
    assert_eq!(2, s.chars().count());
}

fn main() {
    array_search();
    test_bitset();
    test_strings();
}
