use std::ops::{Add, Sub, Mul, Div};
use std::result;

#[derive(Debug, PartialEq)]
pub struct Complex {
    real: f64,
    imaginary: f64,
}

pub type Result<'a> = result::Result<Complex, &'a str>;

impl Complex {
    pub fn new(real: f64, imaginary: f64) -> Self {
        Complex {
            real: real,
            imaginary: imaginary,
        }
    }

    pub fn sqrt(&self) -> Result {
        // see https://en.wikipedia.org/wiki/Complex_number#Square_root

        if self.imaginary == 0.0 {
            return Err("Imaginary part is zero");
        }

        let k = (self.real * self.real + self.imaginary * self.imaginary).sqrt();

        let gamma = ((self.real + k) * 0.5).sqrt();

        let sign = self.imaginary.signum();
        let delta = sign * ((k - self.real) * 0.5).sqrt();

        Ok(Complex {
            real: gamma,
            imaginary: delta,
        })
    }
}

impl Add for Complex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Complex {
            real: self.real + other.real,
            imaginary: self.imaginary + other.imaginary,
        }
    }
}

impl Sub for Complex {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Complex {
            real: self.real - other.real,
            imaginary: self.imaginary - other.imaginary,
        }
    }
}

impl Mul for Complex {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Complex {
            real: self.real * other.real - self.imaginary * other.imaginary,
            imaginary: self.real * other.imaginary + self.imaginary * other.real,
        }
    }
}

impl Div for Complex {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        let divisor = other.real * other.real + other.imaginary * other.imaginary;
        Complex {
            real: (self.real * other.real + self.imaginary * other.imaginary) / divisor,
            imaginary: (self.imaginary * other.real - self.real * other.imaginary) / divisor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 0.0001;

    #[test]
    fn add() {
        assert_eq!(Complex::new(12.0, 14.0),
                   Complex::new(5.0, 2.0) + Complex::new(7.0, 12.0));
        assert_eq!(Complex::new(-2.0, 10.0),
                   Complex::new(5.0, -2.0) + Complex::new(-7.0, 12.0));
    }

    #[test]
    fn subtract() {
        assert_eq!(Complex::new(-2.0, -10.0),
                   Complex::new(5.0, 2.0) - Complex::new(7.0, 12.0));
        assert_eq!(Complex::new(12.0, -14.0),
                   Complex::new(5.0, -2.0) - Complex::new(-7.0, 12.0));
    }

    #[test]
    fn multiply() {
        assert_eq!(Complex::new(11.0, 74.0),
                   Complex::new(5.0, 2.0) * Complex::new(7.0, 12.0));
        assert_eq!(Complex::new(-11.0, 74.0),
                   Complex::new(5.0, -2.0) * Complex::new(-7.0, 12.0));
    }

    #[test]
    fn divide() {
        assert_eq_complex(Complex::new(0.305699482, -0.238341969),
                          Complex::new(5.0, 2.0) / Complex::new(7.0, 12.0));

        assert_eq_complex(Complex::new(-0.305699482, -0.238341969),
                          Complex::new(5.0, -2.0) / Complex::new(-7.0, 12.0));
    }

    #[test]
    fn sqrt() {
        assert_eq_complex_results(Err("Imaginary part is zero"), Complex::new(1.0, 0.0).sqrt());
        assert_eq_complex_results(Ok(Complex::new(2.27872385, -0.438842117)),
                                  Complex::new(5.0, -2.0).sqrt());
        assert_eq_complex_results(Ok(Complex::new(0.438842117, 2.27872385)),
                                  Complex::new(-5.0, 2.0).sqrt());
    }

    fn assert_eq_complex(expect: Complex, value: Complex) {
        assert!((expect.real - value.real) < EPSILON);
        assert!((expect.imaginary - value.imaginary) < EPSILON);
    }

    fn assert_eq_complex_results(expect: Result, value: Result) {
        if expect.is_ok() {
            assert_eq_complex(expect.unwrap(), value.unwrap());
        }
    }
}
