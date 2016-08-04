pub fn base64_encode(input: String) -> String {
    let chars: &[u8] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".as_bytes();

    let data = string_to_vec(input.as_str());
    let mut result: Vec<u8> = vec![];

    let n = input.len();
    let mut i = 0;
    while i < n {
        let char_index = compute_char_index(&data, i);
        let char_indexes = separate_24_to_6_bit(char_index);

        result.push(chars[char_indexes[0]]);
        result.push(chars[char_indexes[1]]);

        if i + 1 < n {
            result.push(chars[char_indexes[2]]);
        }

        if i + 2 < n {
            result.push(chars[char_indexes[3]]);
        }

        i += 3;
    }

    let pads = n % 3;
    if pads > 0 {
        for _ in pads..3 {
            result.push(b'=');
        }
    }

    String::from_utf8(result).unwrap()
}

pub fn base64_decode(input: String) -> String {
    let data = string_to_vec(input.as_str());
    let mut result: Vec<u8> = vec![];

    let mut iter = 0;
    let mut len = 0;

    let mut buf: usize = 0;
    let add_to_result = |result: &mut Vec<u8>, buf: &usize, bits: usize| {
        let value = (buf >> bits) & 255;
        result.push(value as u8);
    };

    let n = data.len();
    let mut i = 0;
    while i < n {
        let c: u8 = BASE64_DECODE_TABLE[data[i]];
        i += 1;

        match c {
            WHITESPACE => continue,
            EQUALS => break,
            _ => {
                buf = (buf << 6) | (c as usize);
                iter += 1;

                if iter == 4 {
                    len += 3;
                    add_to_result(&mut result, &buf, 16);
                    add_to_result(&mut result, &buf, 8);
                    add_to_result(&mut result, &buf, 0);
                    buf = 0;
                    iter = 0;
                }
            }
        }
    }

    if iter == 3 {
        len += 2;
        add_to_result(&mut result, &buf, 10);
        add_to_result(&mut result, &buf, 2);
    } else if iter == 2 {
        len += 1;
        add_to_result(&mut result, &buf, 4);
    }

    result.truncate(len);
    String::from_utf8(result).unwrap()
}

fn compute_char_index(data: &Vec<usize>, i: usize) -> usize {
    let n = data.len();

    let mut char_index = data[i] << 16;

    if i + 1 < n {
        char_index += data[i + 1] << 8;
    }

    if i + 2 < n {
        char_index += data[i + 2];
    }

    char_index
}

fn separate_24_to_6_bit(n: usize) -> [usize; 4] {
    [
        (n >> 18) & 63,
        (n >> 12) & 63,
        (n >> 6) & 63,
        n & 63
    ]
}

fn string_to_vec(input: &str) -> Vec<usize> {
    let mut data: Vec<usize> = vec![];
    for i in input.as_bytes() {
        data.push(*i as usize);
    }
    data
}


const WHITESPACE: u8 = 64u8;
const EQUALS: u8 = 65u8;

const BASE64_DECODE_TABLE: [u8; 256] = [
    66,66,66,66,66,66,66,66,66,66,64,66,66,66,66,66,66,66,66,66,66,66,66,66,66,
    66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,62,66,66,66,63,52,53,
    54,55,56,57,58,59,60,61,66,66,66,65,66,66,66, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,66,66,66,66,66,66,26,27,28,
    29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,66,66,
    66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,
    66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,
    66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,
    66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,
    66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,66,
    66,66,66,66,66,66
];

#[cfg(test)]
mod tests {
    use super::*;

    const RAW: &'static str = "Hello Rust! Привет Раст!\n";
    const ENCODED: &'static str = "SGVsbG8gUnVzdCEg0J/RgNC40LLQtdGCINCg0LDRgdGCIQo=";

    #[test]
    fn encode() {
        assert_eq!(ENCODED.to_string(), base64_encode(RAW.to_string()))
    }

    #[test]
    fn decode() {
        assert_eq!(RAW.to_string(), base64_decode(ENCODED.to_string()))
    }
}
