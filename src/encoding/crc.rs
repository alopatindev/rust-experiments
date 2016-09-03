use std::mem;

pub fn crc(data: u32, key: u32) -> u32 {
    let key_length = leading_one(key);
    let key_shift = key_length - 1;

    let mut data = data;
    data <<= key_shift;

    loop {
        let data_shift = leading_one(data);
        if data_shift <= key_shift {
            break;
        }

        let shifted_key = key << (data_shift - key_length);
        data ^= shifted_key;
    }

    let bits_in_data = mem::size_of_val(&data) * 8;
    let bits_in_data = bits_in_data as u32;

    data <<= bits_in_data - key_shift;
    data >>= bits_in_data - key_shift;

    data
}

pub fn verify(data: u32, key: u32, crc: u32) -> bool {
    let key_length = leading_one(key);
    let crc_length = key_length - 1;

    let mut data = data;
    data <<= crc_length;
    data |= crc;

    loop {
        let data_shift = leading_one(data);
        if data_shift <= crc_length {
            return data == 0;
        }

        let shifted_key = key << (data_shift - key_length);
        data ^= shifted_key;
    }
}

fn leading_one(data: u32) -> u32 {
    let mut data = data;

    for i in 0.. {
        if data == 0 {
            return i;
        }
        data >>= 1;
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        // https://www.youtube.com/watch?v=LL2QpP4k_HE

        let data = 0b1011011;
        let key = 0b1101;
        let crc = crc(data, key);

        assert_eq!(0b001, crc);
        assert!(verify(data, key, crc));

        let corrupted_data = 0b1001011;
        assert_eq!(false, verify(corrupted_data, key, crc));
    }

    quickcheck! {
        fn random_data(data: u32) -> bool {
            let key = 0b1101;
            let crc = crc(data, key);

            if !verify(data, key, crc) {
                return false
            }

            if verify(!data, key, crc) {
                return false
            }

            true
        }
    }

    #[test]
    fn leading_one() {
        assert_eq!(3, super::leading_one(0b101));
        assert_eq!(3, super::leading_one(0b100));
        assert_eq!(4, super::leading_one(0b1100));
        assert_eq!(1, super::leading_one(0b1));
        assert_eq!(0, super::leading_one(0));
    }
}
