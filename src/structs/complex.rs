use std::ops::{Add, Sub, Mul, Div};

#[derive(Debug, PartialEq)]
pub struct Complex {
    real: f64,
    imaginary: f64,
}

impl Complex {
    pub fn new(real: f64, imaginary: f64) -> Self {
        Complex {
            real: real,
            imaginary: imaginary,
        }
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
        let epsilon = 0.0001;
        let result = Complex::new(5.0, 2.0) / Complex::new(7.0, 12.0);
        assert!((0.305699482 - result.real).abs() < epsilon);
        assert!((-0.238341969 - result.imaginary).abs() < epsilon);

        let result = Complex::new(5.0, -2.0) / Complex::new(-7.0, 12.0);
        assert!((-0.305699482 - result.real).abs() < epsilon);
        assert!((-0.238341969 - result.imaginary).abs() < epsilon);
    }
}
