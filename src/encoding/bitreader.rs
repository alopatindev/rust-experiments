use byteorder::{BigEndian, ReadBytesExt};
use std::collections::VecDeque;
use std::io::{Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};

pub struct BitReader<R: Read> {
    input: R,
    buffer: [u8; 1],
    position: u8,
    bytes_read: u64,
    queue: BitQueue,
}

type BitQueue = VecDeque<bool>;

impl<R: Read> BitReader<R> {
    pub fn new(input: R) -> Self {
        BitReader {
            input: input,
            buffer: [0],
            position: 0,
            bytes_read: 0,
            queue: VecDeque::with_capacity(64),
        }
    }

    pub fn read_bit(&mut self) -> Result<bool> {
        if let Some(data) = self.queue.pop_front() {
            return Ok(data);
        }

        if self.position == 0 {
            let new_bytes_read = try!(self.input.read(&mut self.buffer));
            self.bytes_read += new_bytes_read as u64;
            if new_bytes_read == 0 {
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
        let mut rollback_queue = VecDeque::with_capacity(8);
        self.read_u8_internal(&mut rollback_queue)
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let mut data = Vec::with_capacity(8);
        let mut rollback_queue = VecDeque::with_capacity(16);

        for _ in 0..2 {
            match self.read_u8_internal(&mut rollback_queue) {
                Ok(byte) => data.push(byte),
                Err(e) => return Err(e),
            }
        }

        let mut cursor = Cursor::new(data);
        cursor.read_u16::<BigEndian>()
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let mut data = Vec::with_capacity(4);
        let mut rollback_queue = VecDeque::with_capacity(32);

        for _ in 0..4 {
            match self.read_u8_internal(&mut rollback_queue) {
                Ok(byte) => data.push(byte),
                Err(e) => return Err(e),
            }
        }

        let mut cursor = Cursor::new(data);
        cursor.read_u32::<BigEndian>()
    }

    pub fn read_u64(&mut self) -> Result<u64> {
        let mut data = Vec::with_capacity(8);
        let mut rollback_queue = VecDeque::with_capacity(64);

        for _ in 0..8 {
            match self.read_u8_internal(&mut rollback_queue) {
                Ok(byte) => data.push(byte),
                Err(e) => return Err(e),
            }
        }

        let mut cursor = Cursor::new(data);
        cursor.read_u64::<BigEndian>()
    }

    pub fn skip_bits(&mut self, bits: u64) -> Result<()> {
        for _ in 0..bits {
            let _ = try!(self.read_bit());
        }

        Ok(())
    }

    pub fn get_ref(&self) -> &R {
        &self.input
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.input
    }

    fn read_u8_internal(&mut self, mut rollback_queue: &mut BitQueue) -> Result<u8> {
        let mut result = 0;

        for shift in 0..8 {
            match self.read_bit() {
                Ok(bit) => {
                    rollback_queue.push_back(bit);
                    if bit {
                        let shifted_one = 1 << shift;
                        result |= shifted_one;
                    }
                }
                Err(e) => {
                    self.queue.append(&mut rollback_queue);
                    return Err(e);
                }
            }
        }

        Ok(result)
    }
}

impl<R: Read + Seek> Seek for BitReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let old_position = self.position;
        self.position = 0;
        self.bytes_read = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(_) => unimplemented!(), // TODO
            SeekFrom::Current(offset) => {
                let buffer_size = self.buffer.len() as i64;
                self.bytes_read = ((self.bytes_read as i64) + offset - buffer_size) as u64;
                try!(self.skip_bits(old_position as u64));
                self.bytes_read
            }
        };
        self.queue.clear();
        self.input.seek(pos)
    }
}

impl<R: Read + Seek> BitReader<R> {
    pub fn position(&self) -> u64 {
        let bytes_fully_read = if self.bytes_read > 0 && self.position > 0 {
            ((self.bytes_read as i64) - 1) as u64
        } else {
            self.bytes_read
        };
        8 * bytes_fully_read + (self.position as u64)
    }

    pub fn set_position(&mut self, position: u64) -> Result<()> {
        if position != self.position() {
            let byte_position = position / 8;
            let bit_offset = position % 8;
            try!(self.seek(SeekFrom::Start(byte_position)));
            try!(self.skip_bits(bit_offset));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};
    use std::mem;
    use super::*;

    quickcheck! {
        fn random_bits(xs: Vec<u8>) -> bool {
            let input_slice = &xs[..];
            let mut reader = BitReader::new(input_slice);

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
            let mut reader = BitReader::new(input_slice);

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
                let mut reader = BitReader::new(&input_bytes[..]);

                for &expect in input_slice {
                    let data = reader.read_u64().unwrap();
                    if expect != data {
                        return false;
                    }
                }
            }

            true
        }

        fn random_u16s(xs: Vec<u16>) -> bool {
            unsafe {
                let input_slice = &xs[..];
                let input_bytes = vec_to_u8_big_endian(input_slice);
                let mut reader = BitReader::new(&input_bytes[..]);

                for &expect in input_slice {
                    let data = reader.read_u16().unwrap();
                    if expect != data {
                        return false;
                    }
                }
            }

            true
        }

        fn random_u32s(xs: Vec<u32>) -> bool {
            unsafe {
                let input_slice = &xs[..];
                let input_bytes = vec_to_u8_big_endian(input_slice);
                let mut reader = BitReader::new(&input_bytes[..]);

                for &expect in input_slice {
                    let data = reader.read_u32().unwrap();
                    if expect != data {
                        return false;
                    }
                }
            }

            true
        }

        fn random_mixed_types(xs: Vec<u8>) -> bool {
            let input_slice = &xs[..];
            let mut reader = BitReader::new(input_slice);
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

        fn random_positions(xs: Vec<u8>, position: u64) -> bool {
            if xs.is_empty() {
                return true;
            }

            let input_slice = &xs[..];
            let mut reader = BitReader::new(Cursor::new(input_slice));
            let mut position = position % (xs.len() as u64);
            reader.set_position(position).unwrap();

            while let Ok(data) = reader.read_bit() {
                let index = position / 8;
                let shift = position % 8;
                let shifted_one = 1 << shift;
                let expect = (xs[index as usize] & shifted_one) > 0;
                if expect != data {
                    return false;
                }

                position += 1;
                reader.set_position(position).unwrap();
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
    fn read_after_too_much_u8() {
        let input_slice = &[1u8, 2];
        let mut reader = BitReader::new(Cursor::new(input_slice));
        assert_eq!(1, reader.read_u8().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert!(reader.read_u8().is_err());
        assert_eq!(true, reader.read_bit().unwrap());
    }

    #[test]
    fn read_after_too_much_u64() {
        let input_slice = &[0u8, 0, 0, 0, 0, 0, 0, 1, 2];
        let mut reader = BitReader::new(Cursor::new(input_slice));
        assert_eq!(1, reader.read_u64().unwrap());
        assert!(reader.read_u64().is_err());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
        for _ in 0..6 {
            assert_eq!(false, reader.read_bit().unwrap());
        }
        assert!(reader.read_bit().is_err());
    }

    #[test]
    fn seek() {
        let input_slice = &[1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let mut reader = BitReader::new(Cursor::new(input_slice));

        // FIXME: test seek return value

        assert_eq!(0, reader.position());
        assert_eq!(1, reader.read_u8().unwrap());
        assert_eq!(8, reader.position());
        assert_eq!(2, reader.read_u8().unwrap());
        assert_eq!(16, reader.position());
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(17, reader.position());
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(18, reader.position());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(19, reader.position());

        reader.seek(SeekFrom::Current(-1)).unwrap();
        assert_eq!(11, reader.position());
        assert_eq!(false, reader.read_bit().unwrap());
        reader.seek(SeekFrom::Current(1)).unwrap();
        assert_eq!(20, reader.position());
        assert_eq!(false, reader.read_bit().unwrap());

        reader.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(1, reader.read_u8().unwrap());
        assert_eq!(2, reader.read_u8().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(19, reader.position());

        reader.seek(SeekFrom::Start(5)).unwrap();
        assert_eq!(6, reader.read_u8().unwrap());
        assert_eq!(5 * 8 + 8, reader.position());

        // TODO
        // reader.seek(SeekFrom::End(-1)).unwrap();
        // assert_eq!(12, reader.read_u8().unwrap());
        // assert_eq!(input_slice.len() as u64 * 8, reader.position());
    }

    #[test]
    fn set_position() {
        let input_slice = &[1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let mut reader = BitReader::new(Cursor::new(input_slice));

        reader.set_position(8).unwrap();
        assert_eq!(8, reader.position());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(9, reader.position());

        reader.set_position(7).unwrap();
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(8, reader.position());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(9, reader.position());
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(10, reader.position());

        reader.set_position(8).unwrap();
        assert_eq!(2, reader.read_u8().unwrap());

        reader.set_position(8).unwrap();
        assert_eq!(2, reader.read_u8().unwrap());

        for i in 0..12 {
            reader.set_position(8 * i).unwrap();
            let data = (i as u8) + 1;
            assert_eq!(data, reader.read_u8().unwrap());
        }
    }

    #[test]
    fn skip_bits() {
        let input_slice = &[1u8, 2, 3];
        let mut reader = BitReader::new(Cursor::new(input_slice));
        reader.skip_bits(7).unwrap();
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
        reader.skip_bits(6).unwrap();
        assert_eq!(3, reader.read_u8().unwrap());
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
