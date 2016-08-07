use std::cmp;

// fn max_sum_contiguous_naive(a: &Vec<i64>) -> i64 {
//    let n = a.len();
//    let mut sum = i64::min_value();
//    for i in 0..n {
//        for j in i..n {
//            let mut new_sum = 0;
//            for k in i..(j+1) {
//                new_sum += a[k];
//            }
//            if new_sum > sum {
//                sum = new_sum;
//            }
//        }
//    }
//    sum
// }

// Kadane's algorithm
pub fn max_sum_contiguous(a: &[i64]) -> i64 {
    if a.is_empty() {
        return 0;
    }

    let mut prev_arr_sum = a[0];
    let mut max_sum = prev_arr_sum;

    for x in &a[1..] {
        let x = *x;
        prev_arr_sum = cmp::max(x, prev_arr_sum + x);
        max_sum = cmp::max(prev_arr_sum, max_sum);
    }

    max_sum
}

pub fn max_sum_non_contiguous<V: Into<Vec<i64>>>(a: V) -> i64 {
    let mut b = a.into();
    b.sort_by(|a, b| b.cmp(a));
    let mut sum = 0;
    for (i, x) in b.iter().enumerate() {
        let x = *x;
        if x <= 0 {
            if i == 0 {
                return x;
            } else {
                break;
            }
        } else {
            sum += x as i64;
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_contiguous() {
        let a: Vec<i64> = vec![1, 2, -4, 3];
        assert_eq!(1 + 2 + 3, max_sum_non_contiguous(a));

        let a: Vec<i64> = vec![-1, -2, -3];
        assert_eq!(-1, max_sum_non_contiguous(a));
    }

    #[test]
    fn contiguous() {
        let a: Vec<i64> = vec![1, 2, -4, 3, 5, 6, 1, -5];
        assert_eq!(3 + 5 + 6 + 1, max_sum_contiguous(&a));
    }

    #[test]
    fn simple() {
        let a: Vec<i64> = vec![1, 2, 3, 4];
        assert_eq!(10, max_sum_contiguous(&a));
        assert_eq!(10, max_sum_non_contiguous(a));

        let a: Vec<i64> = vec![2, -1, 2, 3, 4, -5];
        assert_eq!(10, max_sum_contiguous(&a));
        assert_eq!(11, max_sum_non_contiguous(a));

        let a: Vec<i64> = vec![];
        assert_eq!(0, max_sum_contiguous(&a));
        assert_eq!(0, max_sum_non_contiguous(a));

        let a: Vec<i64> = vec![1];
        assert_eq!(1, max_sum_contiguous(&a));
        assert_eq!(1, max_sum_non_contiguous(a));

        let a: Vec<i64> = vec![-1, -2, -3, -4, -5, -6];
        assert_eq!(-1, max_sum_contiguous(&a));
        assert_eq!(-1, max_sum_non_contiguous(a));

        let a: Vec<i64> = vec![1, -2];
        assert_eq!(1, max_sum_contiguous(&a));
        assert_eq!(1, max_sum_non_contiguous(a));

        let a: Vec<i64> = vec![1, 2, 3];
        assert_eq!(6, max_sum_contiguous(&a));
        assert_eq!(6, max_sum_non_contiguous(a));

        let a: Vec<i64> = vec![-10];
        assert_eq!(-10, max_sum_contiguous(&a));
        assert_eq!(-10, max_sum_non_contiguous(a));

        let a: Vec<i64> = vec![1, -1, -1, -1, -1, 5];
        assert_eq!(5, max_sum_contiguous(&a));
        assert_eq!(6, max_sum_non_contiguous(a));

        let a: Vec<i64> = vec![1, 2, 3, 4];
        assert_eq!(10, max_sum_contiguous(&a));
        assert_eq!(10, max_sum_non_contiguous(a));

        let a: Vec<i64> = vec![-100, -1];
        assert_eq!(-1, max_sum_contiguous(&a));
        assert_eq!(-1, max_sum_non_contiguous(a));
    }
}
