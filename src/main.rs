extern crate algorithms;

fn array_search() {
    let a: Vec<i32> = vec![10, 22, 55, 66, 66, 333, 1234, 6689];
    assert_eq!(algorithms::algs::binary_search::bsearch(&a, &55), Some(2));
}

fn print_list() {
    let x = algorithms::structs::list::Node::new(1, None);
    let y = x.push(2);
    y.print();
    println!("sum={}", y.sum());
}

fn main() {
    array_search();
    print_list();
}
