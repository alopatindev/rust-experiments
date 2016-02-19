extern crate algorithms;

//use algorithms::binary_search;

fn main() {
    /*let x = Node::new(1, None);
    let y = x.push(2);
    y.print();
    println!("sum={}", y.sum());*/
    let a: Vec<i32> = vec![10, 22, 55, 66, 66, 333, 1234, 6689];
    assert_eq!(algorithms::algs::binary_search::bsearch(&a, &55), Some(2));
}
