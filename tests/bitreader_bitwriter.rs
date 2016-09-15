#[macro_use]
extern crate quickcheck;

extern crate rand;
extern crate rust_experiments;

use rand::Rng;
use rust_experiments::encoding::bitreader::BitReader;
use rust_experiments::encoding::bitwriter::BitWriter;
use std::collections::hash_map::HashMap;
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

        writer.flush().unwrap();

        let input_slice = &input_clone[..];
        assert_eq!(input_slice, writer.get_ref().as_slice());

        true
    }

    fn random_positions(xs: Vec<bool>) -> bool {
        let mut writer = BitWriter::new(vec![]);

        let mut pos_to_value: HashMap<u64, bool> = HashMap::with_capacity(xs.len());
        for i in &xs {
            let value = *i;
            let pos = writer.position();
            pos_to_value.insert(pos, value);
            writer.write_bit(value).unwrap();
        }

        writer.flush().unwrap();

        let mut rng = rand::thread_rng();
        let mut positions: Vec<&u64> = pos_to_value.keys().collect();
        rng.shuffle(positions.as_mut_slice());

        let mut reader = BitReader::new(Cursor::new(writer.get_ref().as_slice()));

        for pos in positions {
            let expect = pos_to_value.get(pos).unwrap();
            reader.set_position(*pos).unwrap();
            let data = reader.read_bit().unwrap();
            if *expect != data {
                return false;
            }
        }

        true
    }
}
