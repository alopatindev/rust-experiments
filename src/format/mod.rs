use nalgebra::clamp;

pub fn size_to_human_readable(bytes: f64) -> String {
    let to_postfix = |exponent| {
        match exponent {
            0 => "B",
            1 => "KiB",
            2 => "MiB",
            3 => "GiB",
            4 => "TiB",
            5 => "PiB",
            6 => "EiB",
            7 => "ZiB",
            _ => "YiB",
        }
    };

    let to_string = |bytes, exponent, two_power_ten: f64| {
        let divisor: f64 = two_power_ten.powi(exponent);
        let number: f64 = bytes / divisor;
        let number_integral = number as usize;
        let fractional: bool = (number.ceil() as usize) > number_integral;
        let postfix = to_postfix(exponent as usize);
        if fractional {
            format!("{:.1} {}", number, postfix)
        } else {
            format!("{} {}", number_integral, postfix)
        }
    };

    let two_power_ten: f64 = 1024.0;
    let exponent = bytes.log(two_power_ten) as i32;
    let exponent = clamp(exponent, 0, 8);
    to_string(bytes, exponent, two_power_ten)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let two_power_ten: f64 = 1024.0;
        assert_eq!("123 B".to_string(), size_to_human_readable(123.0));
        assert_eq!("1023 B".to_string(), size_to_human_readable(1023.0));
        assert_eq!("1 KiB".to_string(), size_to_human_readable(two_power_ten));

        assert_eq!("1.5 KiB".to_string(),
                   size_to_human_readable(1.5 * two_power_ten));
        assert_eq!("2 MiB".to_string(),
                   size_to_human_readable(2.0 * two_power_ten.powi(2)));
        assert_eq!("3 GiB".to_string(),
                   size_to_human_readable(3.0 * two_power_ten.powi(3)));
        assert_eq!("4 TiB".to_string(),
                   size_to_human_readable(4.0 * two_power_ten.powi(4)));
        assert_eq!("5 PiB".to_string(),
                   size_to_human_readable(5.0 * two_power_ten.powi(5)));
        assert_eq!("6 EiB".to_string(),
                   size_to_human_readable(6.0 * two_power_ten.powi(6)));
        assert_eq!("7 ZiB".to_string(),
                   size_to_human_readable(7.0 * two_power_ten.powi(7)));
        assert_eq!("8 YiB".to_string(),
                   size_to_human_readable(8.0 * two_power_ten.powi(8)));
    }
}
