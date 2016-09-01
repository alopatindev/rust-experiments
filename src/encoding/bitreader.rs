use std::io::{Read, Result, Error, ErrorKind};

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

        let bit = 1 << self.position;
        let data = (self.buffer[0] & bit) > 0;

        self.position += 1;
        if self.position >= 8 {
            self.position = 0;
        }

        Ok(data)
    }

    pub fn read_byte(&mut self) -> Result<u8> {
        let mut result = 0;

        for i in 0..8 {
            let data = try!(self.read_bit());
            if data {
                let bit = 1 << i;
                result |= bit;
            }
        }

        Ok(result)
    }

    pub fn get_ref(&self) -> &R {
        &self.input
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.input
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

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
        assert_eq!(123, reader.read_byte().unwrap());
    }

    #[test]
    fn middle_byte() {
        let input_slice = &[0b11000101u8, 0b10000010];
        let mut reader = BitReader::new(Cursor::new(input_slice));
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
        assert_eq!(0b01011000, reader.read_byte().unwrap());
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
        assert_eq!(1, reader.read_byte().unwrap());
        assert_eq!(2, reader.read_byte().unwrap());
        assert!(reader.read_bit().is_err());
        assert!(reader.read_byte().is_err());

        let input_slice = &[1u8, 2];
        let mut reader = BitReader::new(Cursor::new(input_slice));
        assert_eq!(1, reader.read_byte().unwrap());
        assert_eq!(false, reader.read_bit().unwrap());
        assert_eq!(true, reader.read_bit().unwrap());
        assert!(reader.read_byte().is_err());
    }
}
