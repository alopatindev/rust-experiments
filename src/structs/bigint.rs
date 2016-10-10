use std::cmp::{max, Ordering};
use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, Rem};

#[derive(Clone, Debug)]
pub struct BigInt {
    negative: bool,
    digits: Digits, // lowest first
}

pub type Digit = i8;
pub type Digits = Vec<Digit>;

const BASE: Digit = 10;

impl BigInt {
    pub fn new(num: i64) -> Self {
        Self::from(num.to_string().as_str())
    }

    pub fn from(text: &str) -> Self {
        if text.is_empty() {
            return Self::zero();
        }

        let negative = match text.chars().next() {
            Some('-') => true,
            _ => false,
        };

        let char_to_digit = |x| x as Digit - '0' as Digit;

        let mut digits: Digits = text.chars()
            .skip(negative as usize)
            .map(char_to_digit)
            .collect();
        digits.reverse();

        BigInt {
                negative: negative,
                digits: digits,
            }
            .normalize()
    }

    pub fn zero() -> Self {
        BigInt {
            negative: false,
            digits: vec![0],
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.digits.len() + 1);

        if self.negative {
            result.push('-');
        }

        for &i in self.digits.iter().rev() {
            let ch = (i + '0' as Digit) as u8 as char;
            result.push(ch);
        }

        result
    }

    pub fn to_i64(&self) -> Result<i64, &str> {
        if self.clone().abs() <= BigInt::new(i64::max_value()) {
            let mut raw_value = 0;
            let mut shift = 1;

            for &digit in &self.digits {
                raw_value += shift * (digit as u64);
                shift *= 10;
            }

            let value = raw_value as i64;
            if self.negative { Ok(-value) } else { Ok(value) }
        } else {
            Err("The number is out of range")
        }
    }

    pub fn negate(self) -> Self {
        BigInt {
            negative: !self.negative,
            digits: self.digits,
        }
    }

    pub fn abs(self) -> Self {
        BigInt {
            negative: false,
            digits: self.digits,
        }
    }

    pub fn inc(&mut self) {
        *self += BigInt::new(1)
    }

    pub fn dec(&mut self) {
        *self -= BigInt::new(1)
    }

    pub fn pow(&self, exponent: i32) -> Self {
        let mut result = Self::new(1);
        let mut value = self.clone();
        let mut power = exponent;

        while power > 0 {
            if power % 2 == 1 {
                result *= value.clone();
            }

            value *= value.clone();
            power /= 2;
        }

        result
    }

    pub fn sqrt(&self) -> Self {
        self.compute_root_newton(2)
    }

    pub fn factorial(&self) -> Self {
        // FIXME: avoid clone?

        let one = BigInt::new(1);
        let mut n = self.clone();
        n.dec();
        let mut result = self.clone();

        while n > one {
            result *= n.clone();
            n.dec();
        }

        result
    }

    fn compute_root_newton(&self, exponent: i32) -> Self {
        // see http://www.cse.wustl.edu/~kjg/cse131/Notes/SquareRoot/sqrt.html

        let epsilon = BigInt::new(1);
        let exponent_big = BigInt::new(exponent as i64);

        let f = |guess: &BigInt| guess.pow(exponent) - self.clone();

        let f_prime = |guess: &BigInt| exponent_big.clone() * guess.pow(exponent - 1);

        let close_enough = |a: &BigInt, b: &BigInt| {
            let diff = (a.clone() - b.clone()).abs();
            diff < epsilon
        };

        let mut guess = BigInt::new(1);

        loop {
            let new_guess = guess.clone() - f(&guess) / f_prime(&guess);
            if close_enough(&new_guess, &guess) {
                break;
            } else {
                guess = new_guess;
            }
        }

        // this hack improves integer precision
        match (guess.clone() * guess.clone()).partial_cmp(self) {
            Some(Ordering::Less) => guess.inc(),
            Some(Ordering::Greater) => guess.dec(),
            _ => (),
        }

        guess
    }

    fn normalize(mut self) -> Self {
        let leading_zeros = self.digits
            .iter()
            .rev()
            .take_while(|&x| *x == 0)
            .count();

        let n = self.digits.len();
        let zeros_only = n == leading_zeros;
        let digits_len = if zeros_only { 1 } else { n - leading_zeros };
        if zeros_only {
            self.negative = false;
        }

        self.digits.truncate(digits_len);
        self
    }

    // FIXME: refactoring
    fn digits_lt(&self, other: &Self) -> bool {
        let n = self.digits.len();
        let m = other.digits.len();
        if n > m {
            return false;
        }

        n < m ||
        self.digits
            .iter()
            .rev()
            .zip(other.digits.iter().rev())
            .skip_while(|&(&a, &b)| a == b)
            .take(1)
            .any(|(a, b)| a < b)
    }

    fn digits_gt(&self, other: &Self) -> bool {
        let n = self.digits.len();
        let m = other.digits.len();
        if n < m {
            return false;
        }

        n > m ||
        self.digits
            .iter()
            .rev()
            .zip(other.digits.iter().rev())
            .skip_while(|&(&a, &b)| a == b)
            .take(1)
            .any(|(a, b)| a > b)
    }

    fn add_positives(self, other: Self) -> Self {
        let n = max(self.digits.len(), other.digits.len());

        let mut digits = Vec::with_capacity(n + 1);

        let mut carry = 0;
        for i in 0..n {
            let a = match self.digits.get(i) {
                Some(&x) => x,
                None => 0,
            };

            let b = match other.digits.get(i) {
                Some(&x) => x,
                None => 0,
            };

            let sum = a + b + carry;
            carry = sum / BASE;
            let digit = sum % BASE;
            digits.push(digit);
        }

        if carry > 0 {
            digits.push(carry);
        }

        BigInt {
                negative: false,
                digits: digits,
            }
            .normalize()
    }

    fn sub_positives(self, other: Self) -> Self {
        let n = max(self.digits.len(), other.digits.len());

        let mut result = BigInt {
            negative: false,
            digits: Vec::with_capacity(n + 1),
        };

        let mut carry = 0;
        for i in 0..n {
            let a = match self.digits.get(i) {
                Some(&x) => x,
                None => 0,
            };

            let b = match other.digits.get(i) {
                Some(&x) => x,
                None => 0,
            };

            let mut digit = a - b - carry;
            if digit < 0 {
                digit += BASE;
                carry = 1;
            } else {
                carry = 0;
            }

            result.digits.push(digit);
        }

        result.normalize()
    }
}

impl PartialEq for BigInt {
    fn eq(&self, other: &Self) -> bool {
        self.negative == other.negative && self.digits == other.digits
    }
}

impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.eq(other) {
            Some(Ordering::Equal)
        } else if self.lt(other) {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
    }

    fn lt(&self, other: &Self) -> bool {
        if self.negative == other.negative {
            if self.negative {
                self.digits_gt(other)
            } else {
                self.digits_lt(other)
            }
        } else {
            self.negative && !other.negative
        }
    }

    fn gt(&self, other: &Self) -> bool {
        if self.negative == other.negative {
            if self.negative {
                self.digits_lt(other)
            } else {
                self.digits_gt(other)
            }
        } else {
            !self.negative && other.negative
        }
    }
}

impl Add for BigInt {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.negative {
            if other.negative {
                (self.negate() + other.negate()).negate()
            } else {
                other - self.negate()
            }
        } else if other.negative {
            self - other.negate()
        } else {
            self.add_positives(other)
        }
    }
}

impl AddAssign for BigInt {
    fn add_assign(&mut self, other: Self) {
        let result = self.clone() + other;
        *self = result;
    }
}

impl Sub for BigInt {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        if other.negative {
            self + other.negate()
        } else if self.negative {
            (self.negate() + other).negate()
        } else if self < other {
            (other - self).negate()
        } else {
            self.sub_positives(other)
        }
    }
}

impl SubAssign for BigInt {
    fn sub_assign(&mut self, other: Self) {
        let result = self.clone() - other;
        *self = result;
    }
}

impl Mul for BigInt {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let n = max(self.digits.len(), other.digits.len());
        let mut result = vec![0; 2 * n];

        for (i, b) in other.digits.iter().enumerate() {
            for (j, a) in self.digits.iter().enumerate() {
                let index = i + j;
                let value = result[index] + a * b;
                let (carry, digit) = (value / BASE, value % BASE);
                result[index] = digit;
                result[index + 1] += carry;
            }
        }

        let negative = self.negative != other.negative;

        BigInt {
                negative: negative,
                digits: result,
            }
            .normalize()
    }
}

impl MulAssign for BigInt {
    fn mul_assign(&mut self, other: Self) {
        let result = self.clone() * other;
        *self = result;
    }
}

impl Div for BigInt {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        // FIXME: avoid clone?

        let mut result = BigInt::zero();

        let mut numenator = self.clone().abs();
        let divisor = other.clone().abs();

        while numenator > divisor {
            numenator -= divisor.clone();
            result.inc();
        }

        let result = if self.negative != other.negative {
            result.negate()
        } else {
            result
        };

        result.normalize()
    }
}

impl Rem for BigInt {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        // FIXME: avoid clone?

        let mut numenator = self.clone().abs();
        let divisor = other.clone().abs();

        while numenator > divisor {
            numenator -= divisor.clone();
        }

        let result = if self.negative != other.negative {
            numenator.negate()
        } else {
            numenator
        };

        result.normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &'static str = "9876543233387652221098";
    const B: &'static str = "1235557896663457779012";
    const C: &'static str = "-9876543233387652221098";
    const D: &'static str = "-1235557896663457779012";

    #[test]
    fn normalize() {
        assert_eq!(vec![2, 0, 1], BigInt::from("0102").digits);
        assert_eq!(vec![2, 0, 1], BigInt::from("00102").digits);
        assert_eq!(vec![2, 0, 1], BigInt::from("-0102").digits);
        assert_eq!(vec![2, 0, 1], BigInt::from("-00102").digits);
        assert_eq!(vec![0], BigInt::from("-0000").digits);
        assert_eq!(BigInt::zero(), BigInt::from("-0000"));
    }

    #[test]
    fn equals() {
        assert_eq!(BigInt::from(A), BigInt::from(A));
        assert!(BigInt::from(A) != BigInt::from(B));
        assert!(BigInt::from(A) != BigInt::from(C));
        assert_eq!(BigInt::from("1"), BigInt::from("01"));
        assert_eq!(BigInt::from("1"), BigInt::from("001"));
    }

    #[test]
    fn compare() {
        assert!(BigInt::from(A) > BigInt::from(B));
        assert!(BigInt::from(B) < BigInt::from(A));
        assert!(BigInt::from(A) >= BigInt::from(B));
        assert!(BigInt::from(B) <= BigInt::from(A));
        assert!(BigInt::from(A) >= BigInt::from(A));
        assert!(BigInt::from(A) <= BigInt::from(A));
        assert!(!(BigInt::from(A) < BigInt::from(B)));
        assert!(!(BigInt::from(B) > BigInt::from(A)));
        assert!(!(BigInt::from(A) <= BigInt::from(B)));
        assert!(!(BigInt::from(B) >= BigInt::from(A)));

        assert!(!(BigInt::from(A) < BigInt::from(A)));
        assert!(!(BigInt::from(A) > BigInt::from(A)));

        assert!(BigInt::from(C) < BigInt::from(D));
        assert!(BigInt::from(D) > BigInt::from(C));
        assert!(BigInt::from(C) <= BigInt::from(D));
        assert!(BigInt::from(D) >= BigInt::from(C));
        assert!(BigInt::from(D) >= BigInt::from(D));
        assert!(BigInt::from(D) <= BigInt::from(D));
        assert!(!(BigInt::from(C) > BigInt::from(D)));
        assert!(!(BigInt::from(D) < BigInt::from(C)));
        assert!(!(BigInt::from(C) >= BigInt::from(D)));
        assert!(!(BigInt::from(D) <= BigInt::from(C)));

        assert!(BigInt::from("2") > BigInt::from("01"));
        assert!(!(BigInt::from("22") < BigInt::from("13")));
        assert!(BigInt::from("22") > BigInt::from("13"));
        assert!(!(BigInt::from("22") <= BigInt::from("13")));
        assert!(BigInt::from("22") >= BigInt::from("13"));
    }

    #[test]
    fn to_string() {
        assert_eq!("9876543233387652221098", BigInt::from(A).to_string());
        assert_eq!("-9876543233387652221098", BigInt::from(C).to_string());
    }

    #[test]
    fn to_i64() {
        assert_eq!(Ok(123456), BigInt::from("123456").to_i64());
        assert_eq!(Ok(-123456), BigInt::from("-123456").to_i64());

        assert_eq!(Err("The number is out of range"), BigInt::from(A).to_i64());
        assert_eq!(Err("The number is out of range"), BigInt::from(C).to_i64());

        assert_eq!(Err("The number is out of range"),
                   BigInt::new(i64::min_value()).to_i64());

        assert_eq!(Ok(-9223372036854775807),
                   BigInt::new(i64::min_value() + 1).to_i64());

        assert_eq!(Ok(9223372036854775807),
                   BigInt::new(i64::max_value()).to_i64());
    }

    #[test]
    fn add() {
        let result = BigInt::from(A) + BigInt::from(B);
        assert_eq!("11112101130051110000110", result.to_string());

        let result = BigInt::from("3") + BigInt::from("12");
        assert_eq!("15", result.to_string());

        let result = BigInt::from("12") + BigInt::from("3");
        assert_eq!("15", result.to_string());

        let result = BigInt::from(A) + BigInt::from(C);
        assert_eq!(BigInt::zero(), result);

        let result = BigInt::from(C) + BigInt::from(A);
        assert_eq!(BigInt::zero(), result);

        let result = BigInt::from("21098") + BigInt::from("-79012");
        assert_eq!("-57914", result.to_string());

        let result = BigInt::from(A) + BigInt::from(D);
        assert_eq!("8640985336724194442086", result.to_string());

        let result = BigInt::from(D) + BigInt::from(A);
        assert_eq!("8640985336724194442086", result.to_string());

        let result = BigInt::from(C) + BigInt::from(C);
        assert_eq!("-19753086466775304442196", result.to_string());

        let mut result = BigInt::new(12);
        result += BigInt::new(3);
        assert_eq!("15", result.to_string());

        result += BigInt::new(0);
        assert_eq!("15", result.to_string());

        result.inc();
        assert_eq!("16", result.to_string());

        result.dec();
        result.dec();
        assert_eq!("14", result.to_string());
    }

    #[test]
    fn subtract() {
        let result = BigInt::from("12") - BigInt::from("1");
        assert_eq!("11", result.to_string());

        let result = BigInt::from("22") - BigInt::from("13");
        assert_eq!("9", result.to_string());

        let result = BigInt::from("13") - BigInt::from("22");
        assert_eq!("-9", result.to_string());

        let result = BigInt::from("21098") - BigInt::from("9012");
        assert_eq!("12086", result.to_string());

        let result = BigInt::from(A) - BigInt::from(B);
        assert_eq!("8640985336724194442086", result.to_string());

        let result = BigInt::from(A) - BigInt::from(C);
        assert_eq!("19753086466775304442196", result.to_string());

        let result = BigInt::from(C) - BigInt::from(A);
        assert_eq!("-19753086466775304442196", result.to_string());

        let result = BigInt::from(A) - BigInt::from(D);
        assert_eq!("11112101130051110000110", result.to_string());

        let result = BigInt::from(D) - BigInt::from(A);
        assert_eq!("-11112101130051110000110", result.to_string());

        let result = BigInt::from(C) - BigInt::from(C);
        assert_eq!(BigInt::zero(), result);

        let mut result = BigInt::from("13");
        result -= BigInt::from("22");
        assert_eq!("-9", result.to_string());
    }

    #[test]
    fn multiply() {
        let result = BigInt::from("12") * BigInt::from("34");
        assert_eq!("408", result.to_string());

        let result = BigInt::from("-12") * BigInt::from("34");
        assert_eq!("-408", result.to_string());

        let result = BigInt::from("12") * BigInt::from("-34");
        assert_eq!("-408", result.to_string());

        let result = BigInt::from("-12") * BigInt::from("-34");
        assert_eq!("408", result.to_string());

        let result = BigInt::from(A) * BigInt::from(B);
        assert_eq!("12203040983750153968618940597242747847995176",
                   result.to_string());

        let result = BigInt::from(A) * BigInt::from(C);
        assert_eq!("-97546106240975420131236017704190212676325604",
                   result.to_string());

        let mut result = BigInt::from("12");
        result *= BigInt::from("34");
        assert_eq!("408", result.to_string());
    }

    #[test]
    fn divide() {
        let result = BigInt::from(A) / BigInt::from(B);
        assert_eq!("7", result.to_string());

        let result = BigInt::from(B) / BigInt::from(A);
        assert_eq!("0", result.to_string());

        let result = BigInt::from(C) / BigInt::from(D);
        assert_eq!("7", result.to_string());

        let result = BigInt::from(D) / BigInt::from(C);
        assert_eq!("0", result.to_string());

        let result = BigInt::from(A) / BigInt::from(C);
        assert_eq!("0", result.to_string());

        let result = BigInt::from(A) / BigInt::from(D);
        assert_eq!("-7", result.to_string());
    }

    #[test]
    fn modulo() {
        let result = BigInt::from("123") % BigInt::from("5");
        assert_eq!("3", result.to_string());

        let result = BigInt::from("-123") % BigInt::from("5");
        assert_eq!("-3", result.to_string());

        let result = BigInt::from("123") % BigInt::from("-5");
        assert_eq!("-3", result.to_string());

        let result = BigInt::from("-123") % BigInt::from("-5");
        assert_eq!("3", result.to_string());

        let result = BigInt::from(A) % BigInt::from(B);
        assert_eq!("1227637956743447768014", result.to_string());
    }

    #[test]
    fn power() {
        let result = BigInt::from(A).pow(1);
        assert_eq!(A, result.to_string());

        let result = BigInt::from(D).pow(1);
        assert_eq!(D, result.to_string());

        let result = BigInt::from(A).pow(2);
        assert_eq!("97546106240975420131236017704190212676325604",
                   result.to_string());

        let result = BigInt::from(D).pow(2);
        assert_eq!("1526603316007427811481975582035535827696144",
                   result.to_string());

        let result = BigInt::from(D).pow(3);
        assert_eq!("-1886206782165597472597196835163953396428178814255660815036529728",
                   result.to_string());

        let result = BigInt::from(A).pow(12);
        assert_eq!(["86150862503439062135831517110804107838236471384551255671013214923",
                    "96719130438844683846707883110429223798881941924251821429999701137",
                    "16399967186582278339472999886689765270395534765625172882030580925",
                    "17932941737418269392645866530708966234043444024094537663809080689",
                    "0496"]
                       .concat(),
                   result.to_string());
    }

    #[test]
    fn negate() {
        let result = BigInt::from(A).negate();
        assert_eq!("-9876543233387652221098", result.to_string());
    }

    #[test]
    fn abs() {
        assert_eq!(BigInt::from(A), BigInt::from(A).abs());
        assert_eq!(BigInt::from(A), BigInt::from(C).abs());
    }

    #[test]
    fn factorial() {
        assert_eq!("1", BigInt::from("1").factorial().to_string());
        assert_eq!("24", BigInt::from("4").factorial().to_string());
        assert_eq!(["38562048236258042173567706592346364061749310959022359027882840327",
                    "63734025751655435606861685885073615340300518330589163475921729322",
                    "62498857766114955245039357760034644709279247692495585280000000000",
                    "000000000000000000000"]
                       .concat(),
                   BigInt::from("128").factorial().to_string());
    }

    #[test]
    fn sqrt() {
        let assert_approx = |expect, input| BigInt::new(expect) == BigInt::new(input);
        assert_approx(1, 1);
        assert_approx(6, 36);
        assert_approx(12, 144);
        assert_approx(30, 900);
        assert_approx(5, 30);
    }

    // TODO: benchmarks?
}
