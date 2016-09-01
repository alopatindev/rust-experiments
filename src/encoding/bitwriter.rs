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
            try!(self.output.write(&self.buffer));
            self.buffer[0] = 0;
        } else {
            self.position += 1;
        }

        Ok(())
    }

    pub fn write_byte(&mut self, data: u8) -> Result<()> {
        if self.position == 0 {
            self.buffer[0] = data;
            try!(self.output.write(&self.buffer));
        } else {
            for i in 0..8 {
                let bit = 1 << i;
                let d = (data & bit) > 0;
                try!(self.write_bit(d));
            }
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
        if self.buffer[0] > 0 {
            let _ = self.output.write(&self.buffer);
        }
        let _ = self.output.flush();
    }
}

impl<T: Write> Drop for BitWriter<T> {
    fn drop(&mut self) {
        self.flush();
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

    #[test]
    fn bits() {
        let mut writer = new_writer(2);
        assert_eq!(0, writer.get_ref().position());

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
        assert_eq!(expect, get_data(writer));
    }

    fn assert_position(expect: u64, writer: &MockWriter) {
        assert_eq!(expect, writer.get_ref().position());
    }

    fn get_data(writer: &MockWriter) -> &[u8] {
        let cursor: &Cursor<Vec<u8>> = writer.get_ref();
        let pos: usize = cursor.position() as usize;
        &cursor.get_ref()[0..pos]
    }
}
