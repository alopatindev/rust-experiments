pub fn base64_encode(input: &'static str) -> String {
    let chars: &[u8] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".as_bytes();

    let mut data: Vec<usize> = vec![];
    for i in input.as_bytes().iter() {
        data.push(*i as usize);
    }

    let data_length = input.len();
    let pad_count = data_length % 3;

    let mut result: Vec<u8> = vec![];
    let mut data_index = 0;
    while data_index < data_length {
        let char_index = compute_char_index(&data, data_index);
        let char_indexes = separate_24_to_6_bit(char_index);

        result.push(chars[char_indexes[0]]);
        result.push(chars[char_indexes[1]]);

        if data_index + 1 < data_length {
            result.push(chars[char_indexes[2]]);
        }

        if data_index + 2 < data_length {
            result.push(chars[char_indexes[3]]);
        }

        data_index += 3;
    }

    if pad_count > 0 {
        for _ in pad_count..3 {
            result.push(b'=');
        }
    }

    String::from_utf8(result).unwrap()
}

fn compute_char_index(data: &Vec<usize>, index: usize) -> usize {
    let len = data.len();

    let mut char_index = data[index] << 16;

    if index + 1 < len {
        char_index += data[index + 1] << 8;
    }

    if index + 2 < len {
        char_index += data[index + 2];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64() {
        let a = "aGVsbG8gcnVzdCE=".to_string();
        let b = "hello rust!";
        assert_eq!(a, base64_encode(b))
    }
}
