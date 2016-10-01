pub struct HuffmanDecoder<R: Read + Seek> {
    input: BitReader<R>,
    code_to_char: HashMap<Code, Char>,
    data_offset_bit: u64,
}

impl<R: Read + Seek> HuffmanDecoder<R> {
    pub fn new(input: R) -> Result<Self> {
        let mut result = HuffmanDecoder {
            input: BitReader::new(input),
            code_to_char: HashMap::new(),
            data_offset_bit: 0,
        };

        if result.read_header().is_err() {
            let e = Error::new(ErrorKind::InvalidInput, "Failed to read the header");
            Err(e)
        } else {
            Ok(result)
        }
    }

    pub fn decode(&mut self,
                  output: &mut Write,
                  offset_bit: u64,
                  original_length_bits: u64)
                  -> Result<u64> {
        let mut read_bytes = 0;

        if original_length_bits > 0 {
            let original_length_bytes = original_length_bits / 8;

            try!(self.input.set_position(offset_bit));

            while read_bytes < original_length_bytes {
                match self.decode_char() {
                    Some(ch) => {
                        try!(output.write(ch));
                        read_bytes += ch.len() as u64;
                    }
                    None => unreachable!(),
                }
            }

            try!(output.flush());
        }

        let read_bits = read_bytes * 8;
        Ok(read_bits)
    }

    pub fn data_offset_bit(&self) -> u64 {
        self.data_offset_bit
    }

    pub fn get_input_mut(&mut self) -> &mut R {
        self.input.get_mut()
    }

    pub fn get_reader_mut(&mut self) -> &mut BitReader<R> {
        &mut self.input
    }

    fn read_header(&mut self) -> Result<()> {
        let dict_length: DictLength = try!(self.input.read_u16());
        let dict_length = dict_length as usize;
        self.code_to_char.reserve(dict_length);

        for _ in 0..dict_length {
            let code_length = try!(self.input.read_u8());
            let code_data = try!(self.input.read_u16());
            let char_length = try!(self.input.read_u8()) as usize;
            match read_char(&mut self.input, char_length) {
                Some(ch) => {
                    let code = Code {
                        length: code_length,
                        data: code_data,
                    };
                    self.code_to_char.insert(code, ch);
                }
                None => unreachable!(),
            }
        }

        self.data_offset_bit = self.input.position();

        Ok(())
    }

    fn decode_char(&mut self) -> Option<CharSlice> {
        let mut code = Code {
            length: 0,
            data: 0,
        };

        while let Ok(data) = self.input.read_bit() {
            assert!(code.length <= max_code_length());

            if data {
                let shifted_one = 1 << code.length;
                code.data |= shifted_one;
            }

            code.length += 1;

            if let Some(ref ch) = self.code_to_char.get(&code) {
                return Some((*ch).as_slice());
            }
        }

        println!("Error: couldn't decode character length={} of {}; code={:b}; dict len={}",
                 code.length,
                 max_code_length(),
                 code.data,
                 self.code_to_char.len());

        None
    }
}
