use byteorder::{ByteOrder, BigEndian};
use std::io::{Write, Result};
use std::ops::Drop;

pub struct BitWriter<W: Write> {
    output: W,
    buffer: [u8; 1],
    position: u8,
}

impl<W: Write> BitWriter<W> {
    pub fn new(output: W) -> Self {
        BitWriter {
            output: output,
            buffer: [0],
            position: 0,
        }
    }

    pub fn write_bit(&mut self, data: bool) -> Result<()> {
        if data {
            let bit = 1 << self.position;
            self.buffer[0] |= bit;
        }

        if self.position >= 7 {
            self.position = 0;
            try!(self.output.write_all(&self.buffer));
            self.buffer[0] = 0;
        } else {
            self.position += 1;
        }

        Ok(())
    }

    pub fn write_byte(&mut self, data: u8) -> Result<()> {
        for i in 0..8 {
            let bit = 1 << i;
            let d = (data & bit) > 0;
            try!(self.write_bit(d));
        }

        Ok(())
    }

    pub fn write_u64(&mut self, data: u64) -> Result<()> {
        let mut buffer = [0; 8];
        BigEndian::write_u64(&mut buffer, data);
        for &i in &buffer {
            try!(self.write_byte(i));
        }

        Ok(())
    }

    pub fn get_ref(&self) -> &W {
        &self.output
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.output
    }

    pub fn flush(&mut self) {
        if self.position != 0 {
            let _ = self.output.write_all(&self.buffer);
            self.position = 0;
        }
        self.output.flush().unwrap();
    }
}

impl<T: Write> Drop for BitWriter<T> {
    fn drop(&mut self) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use byteorder::{BigEndian, ReadBytesExt};
    use std::io::Cursor;
    use std::mem;
    use super::*;

    #[quickcheck]
    fn random_bits(xs: Vec<u8>) -> bool {
        let mut writer = new_writer(xs.len());
        for &i in &xs {
            for shift in 0..8 {
                let bit = 1 << shift;
                let data = (i & bit) > 0;
                writer.write_bit(data).unwrap();
            }
        }
        writer.flush();
        check_u8_data(&xs[..], &writer)
    }

    #[quickcheck]
    fn random_bytes(xs: Vec<u8>) -> bool {
        let mut writer = new_writer(xs.len());
        for &i in &xs {
            writer.write_byte(i).unwrap();
        }
        writer.flush();
        check_u8_data(&xs[..], &writer)
    }

    #[quickcheck]
    fn random_u64s(xs: Vec<u64>) -> bool {
        let mut writer = new_writer(xs.len());
        for &i in &xs {
            writer.write_u64(i).unwrap();
        }
        writer.flush();
        check_u64_data(&xs[..], &writer)
    }

    #[quickcheck]
    fn random_mixed_types(xs: Vec<u8>) -> bool {
        let mut writer = new_writer(xs.len());
        let mut bytes = true;

        for &i in &xs {
            if bytes {
                writer.write_byte(i).unwrap();
            } else {
                for shift in 0..8 {
                    let bit = 1 << shift;
                    let data = (i & bit) > 0;
                    writer.write_bit(data).unwrap();
                }
            }
            bytes = !bytes;
        }
        writer.flush();

        check_u8_data(&xs[..], &writer)
    }

    #[test]
    fn bits() {
        let mut writer = new_writer(2);
        assert_position(0, &writer);

        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        assert_position(0, &writer);
        writer.write_bit(false).unwrap();
        assert_position(0, &writer);
        writer.write_bit(false).unwrap();
        assert_position(1, &writer);
        assert_data(&[0b00111001], &writer);

        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(false).unwrap();
        assert_position(1, &writer);
        writer.write_bit(false).unwrap();
        assert_position(2, &writer);

        assert_data(&[0b00111001, 0b00001111], &writer);
    }

    #[test]
    fn two_bytes() {
        let mut writer = new_writer(2);
        assert_position(0, &writer);
        writer.write_byte(12).unwrap();
        assert_position(1, &writer);
        writer.write_byte(34).unwrap();
        assert_position(2, &writer);
        assert_data(&[12, 34], &writer);
    }

    #[test]
    fn middle_byte() {
        let mut writer = new_writer(3);
        writer.write_bit(true).unwrap();
        writer.write_bit(false).unwrap();
        writer.write_bit(true).unwrap();
        assert_position(0, &writer);
        writer.write_byte(1).unwrap();
        assert_position(1, &writer);
        writer.write_bit(true).unwrap();
        assert_position(1, &writer);
        assert_data(&[0b00001101], &writer);
        writer.flush();
        assert_data(&[0b00001101, 0b00001000], &writer);
    }

    type MockWriter = BitWriter<Cursor<Vec<u8>>>;

    fn new_writer(size: usize) -> MockWriter {
        let buffer: Vec<u8> = vec![0; size];
        let cursor = Cursor::new(buffer);
        BitWriter::new(cursor)
    }

    fn assert_data(expect: &[u8], writer: &MockWriter) {
        assert_eq!(expect, get_u8_data(writer));
    }

    fn check_u8_data(expect: &[u8], writer: &MockWriter) -> bool {
        expect == get_u8_data(writer)
    }

    fn check_u64_data(expect: &[u64], writer: &MockWriter) -> bool {
        expect == &get_u64_data(writer)[..]
    }

    fn assert_position(expect: u64, writer: &MockWriter) {
        assert_eq!(expect, writer.get_ref().position());
    }

    fn get_u8_data(writer: &MockWriter) -> &[u8] {
        let cursor: &Cursor<Vec<u8>> = writer.get_ref();
        let pos = cursor.position() as usize;
        &cursor.get_ref()[0..pos]
    }

    fn get_u64_data(writer: &MockWriter) -> Vec<u64> {
        let cursor: &Cursor<Vec<u8>> = writer.get_ref();
        let pos = cursor.position() as usize;
        let bytes_per_item = mem::size_of::<u64>();

        let mut result = Vec::with_capacity(pos / bytes_per_item);
        let mut i = 0;
        while i < pos {
            let data = &cursor.get_ref()[i..(i + bytes_per_item)];
            let mut cursor = Cursor::new(data);
            let item = cursor.read_u64::<BigEndian>().unwrap();
            result.push(item);
            i += bytes_per_item;
        }

        result
    }
}
