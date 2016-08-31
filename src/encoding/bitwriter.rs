use std::io::{Write, Result};
use std::ops::Drop;

pub struct BitWriter<'a, W: Write + 'a> {
    output: &'a mut W,
    buffer: [u8; 1],
    position: u8,
}

impl<'a, W: Write> BitWriter<'a, W> {
    pub fn new(output: &'a mut W) -> Self {
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
        self.output
    }

    pub fn get_mut(&mut self) -> &mut W {
        self.output
    }

    pub fn flush(&mut self) {
        if self.buffer[0] > 0 {
            let _ = self.output.write(&self.buffer);
        }
        let _ = self.output.flush();
    }
}

impl<'a, T: Write> Drop for BitWriter<'a, T> {
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
        let size = 2;
        let buffer_vec: Vec<u8> = vec![0; size];
        let mut buffer = Cursor::new(buffer_vec);
        let mut writer = BitWriter::new(&mut buffer);
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
        let size = 2;
        let buffer_vec: Vec<u8> = vec![0; size];
        let mut buffer = Cursor::new(buffer_vec);
        let mut writer = BitWriter::new(&mut buffer);
        assert_position(0, &writer);
        writer.write_byte(12).unwrap();
        assert_position(1, &writer);
        writer.write_byte(34).unwrap();
        assert_position(2, &writer);
        assert_data(&[12, 34], &writer);
    }

    #[test]
    fn middle_byte() {
        let size = 3;
        let buffer_vec: Vec<u8> = vec![0; size];
        let mut buffer = Cursor::new(buffer_vec);
        let mut writer = BitWriter::new(&mut buffer);
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

    #[test]
    fn after_drop() {
        let size = 2;
        let buffer_vec: Vec<u8> = vec![0; size];
        let mut buffer = Cursor::new(buffer_vec);
        {
            let mut writer = BitWriter::new(&mut buffer);
            writer.write_byte(12).unwrap();
            writer.write_bit(true).unwrap();
            writer.write_bit(false).unwrap();
            writer.write_bit(true).unwrap();
            writer.write_bit(true).unwrap();
            assert_position(1, &writer);
        }
        assert_eq!(vec![12, 0b00001101], *buffer.get_ref());
    }

    fn assert_data<'a>(expect: &[u8], writer: &'a BitWriter<'a, Cursor<Vec<u8>>>) {
        assert_eq!(expect, get_data(writer));
    }

    fn assert_position<'a>(expect: u64, writer: &'a BitWriter<'a, Cursor<Vec<u8>>>) {
        assert_eq!(expect, writer.get_ref().position());
    }

    fn get_data<'a>(writer: &'a BitWriter<'a, Cursor<Vec<u8>>>) -> &'a [u8] {
        let cursor: &'a Cursor<Vec<u8>> = writer.get_ref();
        let pos: usize = cursor.position() as usize;
        &cursor.get_ref()[0..pos]
    }
}
