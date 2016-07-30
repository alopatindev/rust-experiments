pub fn to_utf8(utf16: &[u16]) -> Vec<u8> {
    let mut utf8 = vec![];

    for ch in utf16 {
        if *ch <= 0x7f {
            utf8.push(*ch as u8)
        } else if *ch <= 0x7ff {
            utf8.push(0b11000000 | (*ch >> 6) as u8);
            utf8.push(0b10000000 | ((*ch << 10) >> 10) as u8);
        } else {
            utf8.push(0b11100000 | (*ch >> 12) as u8);
            utf8.push(0b10000000 | ((*ch << 4) >> 10) as u8);
            utf8.push(0b10000000 | ((*ch << 10) >> 10) as u8);
        }
    }

    utf8
}

pub fn to_utf16(utf8: &[u8]) -> Vec<u16> {
    let mut utf16: Vec<u16> = vec![];
    let mut i = 0;

    let mut ch;
    while i < utf8.len() {
        if (utf8[i] >> 7) == 0 {
            ch = utf8[i] as u16;
            i += 1;
        } else if (utf8[i] >> 5) == 0b110 {
            let high = ((utf8[i] & 0b00111111) as u16) << 6;
            let low = (utf8[i + 1] & 0b01111111) as u16;
            ch = high | low;
            i += 2;
        } else {
            let a = ((utf8[i] & 0b00011111) as u16) << 12;
            let b = ((utf8[i + 1] & 0b01111111) as u16) << 6;
            let c = (utf8[i + 2] & 0b01111111) as u16;
            ch = a | b | c;
            i += 3;
        }

        utf16.push(ch);
    }

    utf16
}

#[cfg(test)]
mod tests {
    pub fn make_strings(utf8: &str) -> (Vec<u8>, Vec<u16>) {
        let mut utf8_bytes: Vec<u8> = vec![];
        for i in utf8.as_bytes() {
            utf8_bytes.push(*i);
        }

        let utf16: Vec<u16> = utf8.to_string().encode_utf16().collect();
        (utf8_bytes, utf16)
    }

    mod utf16_to_utf8 {
        use super::*;
        use super::super::*;

        #[test]
        fn test_ascii() {
            let (utf8, utf16) = make_strings("Hello, Rust!");
            assert_eq!(utf8, to_utf8(&utf16[..]));
        }

        #[test]
        fn test_russian() {
            let (utf8, utf16) = make_strings("Привет, Раст!");
            assert_eq!(utf8, to_utf8(&utf16[..]));
        }

        #[test]
        fn test_japanese() {
            let (utf8, utf16) = make_strings("こんにちは、さび");
            assert_eq!(utf8, to_utf8(&utf16[..]));
        }

        #[test]
        fn test_bounds() {
            let (utf8, utf16) = make_strings("\u{0000}\u{007f}");
            assert_eq!(utf8, to_utf8(&utf16[..]));

            let (utf8, utf16) = make_strings("\u{0080}\u{07ff}");
            assert_eq!(utf8, to_utf8(&utf16[..]));

            let (utf8, utf16) = make_strings("\u{0800}\u{ffff}");
            assert_eq!(utf8, to_utf8(&utf16[..]));
        }
    }

    mod utf8_to_utf16 {
        use super::*;
        use super::super::*;

        #[test]
        fn test_ascii() {
            let (utf8, utf16) = make_strings("Hello, Rust!");
            assert_eq!(utf16, to_utf16(&utf8[..]));
        }

        #[test]
        fn test_russian() {
            let (utf8, utf16) = make_strings("Привет, Раст!");
            assert_eq!(utf16, to_utf16(&utf8[..]));
        }

        #[test]
        fn test_japanese() {
            let (utf8, utf16) = make_strings("こんにちは、さび");
            assert_eq!(utf16, to_utf16(&utf8[..]));
        }

        #[test]
        fn test_bounds() {
            let (utf8, utf16) = make_strings("\u{0000}\u{007f}");
            assert_eq!(utf16, to_utf16(&utf8[..]));

            let (utf8, utf16) = make_strings("\u{0080}\u{07ff}");
            assert_eq!(utf16, to_utf16(&utf8[..]));

            let (utf8, utf16) = make_strings("\u{0800}\u{ffff}");
            assert_eq!(utf16, to_utf16(&utf8[..]));
        }
    }
}
