pub fn quick_sort<T>(a: &mut Vec<T>) -> &mut Vec<T>
    where T: PartialOrd + Copy
{
    unimplemented!()
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
