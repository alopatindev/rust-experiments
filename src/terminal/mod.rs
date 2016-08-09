extern crate terminal_size;

use self::terminal_size::{Width, terminal_size};

pub fn clear_current_line() {
    print!("\r");

    let width = if let Some((Width(w), _)) = terminal_size() {
        w
    } else {
        80
    };

    for _ in 0..width {
        print!(" ");
    }

    print!("\r");
}

pub fn progress_bar(progress: f64, length: i32) -> String {
    assert!(progress >= 0.0 && progress <= 1.0);

    let brackets_length = 2;
    assert!(length >= brackets_length);

    let mut result = String::with_capacity(length as usize);

    let length = length - brackets_length;
    let progress_length = progress * length as f64;
    let progress_length = progress_length as i32;

    result.push('[');

    for i in 1..(length + 1) {
        let next_char = if i < progress_length {
            '='
        } else if i == progress_length {
            '>'
        } else {
            ' '
        };
        result.push(next_char);
    }

    result.push(']');

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar() {
        assert_eq!("[        ]".to_string(), progress_bar(0.0, 10));
        assert_eq!("[=>      ]".to_string(), progress_bar(0.3, 10));
        assert_eq!("[==>     ]".to_string(), progress_bar(0.4, 10));
        assert_eq!("[===>    ]".to_string(), progress_bar(0.5, 10));
        assert_eq!("[=======>]".to_string(), progress_bar(1.0, 10));
    }

    #[test]
    #[should_panic]
    fn too_big_progress() {
        let _ = progress_bar(2.0, 10);
    }

    #[test]
    #[should_panic]
    fn too_small_progress() {
        let _ = progress_bar(-2.0, 10);
    }
}
