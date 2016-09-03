use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};

pub struct BitReader<R: Read> {
    input: R,
    buffer: [u8; 1],
    position: u8,
}

impl<R: Read> BitReader<R> {
    pub fn new(input: R) -> BitReader<R> {
        BitReader {
            input: input,
            buffer: [0],
            position: 0,
        }
    }

    pub fn read_bit(&mut self) -> Result<bool> {
        if self.position == 0 {
            let bytes_read = try!(self.input.read(&mut self.buffer));
            if bytes_read == 0 {
                let e = Error::new(ErrorKind::UnexpectedEof, "No more data");
                return Err(e);
            }
        }

        let shifted_one = 1 << self.position;
        let data = (self.buffer[0] & shifted_one) > 0;

        self.position += 1;
        if self.position >= 8 {
            self.position = 0;
        }

        Ok(data)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let mut result = 0;

        for i in 0..8 {
            let data = try!(self.read_bit());
            if data {
                let shifted_one = 1 << i;
                result |= shifted_one;
            }
        }

        Ok(result)
    }

    pub fn read_u64(&mut self) -> Result<u64> {
        let mut data = Vec::with_capacity(8);

        for _ in 0..8 {
            let byte = try!(self.read_u8());
            data.push(byte);
        }

        let mut cursor = Cursor::new(data);
        cursor.read_u64::<BigEndian>()
    }

    pub fn get_ref(&self) -> &R {
        &self.input
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.input
    }
}

impl<R: Read + Seek> Seek for BitReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        // FIXME: maintain the state
        self.input.seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::mem;
    use super::*;

    quickcheck! {
        fn random_bits(xs: Vec<u8>) -> bool {
            let input_slice = &xs[..];
            let mut reader = BitReader::new(Cursor::new(input_slice));

            for &i in input_slice {
                for shift in 0..8 {
                    let shifted_one = 1 << shift;
                    let expect = (i & shifted_one) > 0;
                    let data = reader.read_bit().unwrap();
                    if expect != data {
                        return false;
                    }
                }
            }

            true
        }

        fn random_bytes(xs: Vec<u8>) -> bool {
            let input_slice = &xs[..];
            let mut reader = BitReader::new(Cursor::new(input_slice));

            for &expect in input_slice {
                let data = reader.read_u8().unwrap();
                if expect != data {
                    return false;
                }
            }

            true
        }

        fn random_u64s(xs: Vec<u64>) -> bool {
            unsafe {
                let input_slice = &xs[..];
                let input_bytes = vec_to_u8_big_endian(input_slice);
                let mut reader = BitReader::new(Cursor::new(&input_bytes[..]));

                for &expect in input_slice {
                    let data = reader.read_u64().unwrap();
                    if expect != data {
                        return false;
                    }
                }
            }

            true
        }

        fn random_mixed_types(xs: Vec<u8>) -> bool {
            let input_slice = &xs[..];
            let mut reader = BitReader::new(Cursor::new(input_slice));
            let mut bytes = true;

            for &i in input_slice {
                if bytes {
                    let expect = i;
                    let data = reader.read_u8().unwrap();
                    if expect != data {
                        return false;
                    }
                } else {
                    for shift in 0..8 {
                        let shifted_one = 1 << shift;
                        let expect = (i & shifted_one) > 0;
                        let data = reader.read_bit().unwrap();
                        if expect != data {
                            return false;
                        }
                    }
                }
                bytes = !bytes;
            }

            true
        }
    }

    #[test]
    fn simple() {
        let input_slice = &[0b11000101u8, 123];
        let mut reader = BitReader::new(Cursor::new(input_slice));
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(123, reader.read_u8().unwrap());
    }

    #[test]
    fn middle_byte() {
        let input_slice = &[0b11000101u8, 0b10000010];
        let mut reader = BitReader::new(Cursor::new(input_slice));
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(0b01011000, reader.read_u8().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
    }

    #[test]
    fn read_too_much() {
        let input_slice = &[1u8, 2];
        let mut reader = BitReader::new(Cursor::new(input_slice));
        assert_eq!(1, reader.read_u8().unwrap());
        assert_eq!(2, reader.read_u8().unwrap());
        assert!(reader.read_bit().is_err());
        assert!(reader.read_u8().is_err());

        let input_slice = &[1u8, 2];
        let mut reader = BitReader::new(Cursor::new(input_slice));
        assert_eq!(1, reader.read_u8().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
        assert!(reader.read_u8().is_err());
    }

    #[test]
    fn test_vec_to_u8() {
        unsafe {
            let xs = vec![13u64];
            assert_eq!(&[0, 0, 0, 0, 0, 0, 0, 13],
                       vec_to_u8_big_endian(&xs).as_slice());

            let xs = vec![1u64, 2];
            assert_eq!(&[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2],
                       vec_to_u8_big_endian(&xs).as_slice());
        }
    }

    unsafe fn vec_to_u8_big_endian<T>(input: &[T]) -> Vec<u8> {
        let bytes_per_item = mem::size_of::<T>();
        let n = input.len() * bytes_per_item;
        let mut result = Vec::with_capacity(n);

        let data: *const T = input.as_ptr();
        let data: *const u8 = mem::transmute(data);

        let mut i = 0;
        while i < n {
            let mut bytes_indexes = (0..bytes_per_item).collect::<Vec<_>>();
            if cfg!(target_endian = "little") {
                bytes_indexes.reverse();
            }

            let mut item = Vec::with_capacity(bytes_per_item);
            for &offset in &bytes_indexes[..] {
                let offs = i + offset;
                let offs = offs as isize;
                let p: *const u8 = data.offset(offs);
                item.push(*p);
            }

            assert_eq!(bytes_per_item, item.len());
            result.append(&mut item);

            i += bytes_per_item;
        }

        result
    }
}
