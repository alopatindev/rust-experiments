#[macro_use]
extern crate quickcheck;

extern crate rust_experiments;

use rust_experiments::encoding::bitreader::BitReader;
use rust_experiments::encoding::bitwriter::BitWriter;
use std::io::Cursor;

quickcheck! {
    fn simple(input: Vec<u8>) -> bool {
        let input_clone = input.clone();

        let mut reader = BitReader::new(Cursor::new(input));
        let mut writer = BitWriter::new(vec![]);

        loop {
            if let Ok(long) = reader.read_u64() {
                writer.write_u64(long).unwrap();
            }

            if let Ok(byte) = reader.read_u8() {
                writer.write_u8(byte).unwrap();
            }

            match reader.read_bit() {
                Ok(bit) => writer.write_bit(bit).unwrap(),
                Err(_) => break,
            }
        }

        writer.flush();

        let input_slice = &input_clone[..];
        assert_eq!(input_slice, &writer.get_ref()[..]);

        true
    }
}
