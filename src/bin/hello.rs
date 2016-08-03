extern crate rust_experiments;
use rust_experiments::*;

fn array_search() {
    let a: Vec<i32> = vec![10, 22, 55, 66, 66, 333, 1234, 6689];
    assert_eq!(algorithms::binary_search::binary_search(&a, &55), Some(2));
}

fn print_list() {
    let x = structs::list::Node::new(1, None);
    let y = x.push(2);
    y.print();
    println!("sum={}", y.sum());
}

fn test_bitset() {
    let a = vec![10, 22, 55, 66, 66, 333, 1234, 6689];
    let mut set: structs::bitset::BitSet = structs::bitset::BitSet::new();
    for i in &a {
        set.set(*i);
    }
    println!("set={}", set);

    let unset = a[1];
    set.clear(unset);
    println!("set={} with unset={}", set, unset);
}

fn main() {
    array_search();
    print_list();
    test_bitset();
}
