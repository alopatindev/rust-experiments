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

    let pad_count = n % 3;
    if pad_count > 0 {
        for _ in pad_count..3 {
            result.push(b'=');
        }
    }

    String::from_utf8(result).unwrap()
}

pub fn base64_decode(input: String) -> String {
    input.to_string() // TODO
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

#[cfg(test)]
mod tests {
    use super::*;

    const RAW: &'static str = "hello rust!";
    const ENCODED: &'static str = "aGVsbG8gcnVzdCE=";

    #[test]
    fn test_base64_encode() {
        assert_eq!(ENCODED.to_string(), base64_encode(RAW.to_string()))
    }

    #[test]
    fn test_base64_decode() {
        assert_eq!(RAW.to_string(), base64_decode(ENCODED.to_string()))
    }
}
