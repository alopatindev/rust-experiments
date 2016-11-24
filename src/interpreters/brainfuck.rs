use std::io::{Read, Write};
use std::num::Wrapping;

pub struct VM<R: Read, W: Write> {
    program: Vec<char>,
    input: R,
    output: W,
    output_buffer: Vec<u8>,

    memory: Vec<u8>,
    memory_index: usize,
    program_counter: usize,

    program_counter_updated: bool,
    loops: Vec<usize>,
}

const MEMORY_CAPACITY: usize = 512;
const OUTPUT_BUFFER_CAPACITY: usize = 128;
const LOOPS_CAPACITY: usize = 128;

impl<R: Read, W: Write> VM<R, W> {
    pub fn new(program: String, input: R, output: W) -> Self {
        let supported_commands = hashset!{'+', '-', '<', '>', '[', ']', ',', '.'};
        let filtered_program = program.chars()
            .filter(|x| supported_commands.contains(x))
            .collect::<Vec<char>>();

        VM {
            program: filtered_program,
            input: input,
            output: output,
            output_buffer: Vec::with_capacity(OUTPUT_BUFFER_CAPACITY),

            memory: Vec::with_capacity(MEMORY_CAPACITY),
            memory_index: 0,
            program_counter: 0,

            program_counter_updated: false,
            loops: Vec::with_capacity(LOOPS_CAPACITY),
        }
    }

    pub fn run(&mut self) {
        while self.program_counter < self.program.len() {
            let command = self.program[self.program_counter];
            match command {
                '+' => self.increment(),
                '-' => self.decrement(),
                '<' => self.shift_left(),
                '>' => self.shift_right(),
                '[' => self.loop_begin(),
                ']' => self.loop_end(),
                ',' => self.read(),
                '.' => self.write(),
                _ => unreachable!(),
            }

            if self.program_counter_updated {
                self.program_counter_updated = false;
            } else {
                self.program_counter += 1;
            }
        }

        if !self.output_buffer.is_empty() {
            self.flush_output_buffer();
        }
    }

    pub fn get_output_mut(&mut self) -> &mut W {
        &mut self.output
    }

    pub fn filtered_memory(&self) -> Vec<u8> {
        let mut postfix_zeros = self.memory.len();
        for &value in self.memory.iter().rev() {
            if value == 0 {
                postfix_zeros -= 1;
            } else {
                break;
            }
        }

        self.memory[0..postfix_zeros].to_vec()
    }

    fn increment(&mut self) {
        self.maybe_grow_memory();
        let new_value = Wrapping(self.memory[self.memory_index]) + Wrapping(1);
        self.memory[self.memory_index] = new_value.0;
    }

    fn decrement(&mut self) {
        self.maybe_grow_memory();
        let new_value = Wrapping(self.memory[self.memory_index]) - Wrapping(1);
        self.memory[self.memory_index] = new_value.0;
    }

    fn shift_left(&mut self) {
        self.memory_index -= 1;
        self.maybe_grow_memory();
    }

    fn shift_right(&mut self) {
        self.memory_index += 1;
        self.maybe_grow_memory();
    }

    fn loop_begin(&mut self) {
        self.maybe_grow_memory();
        if self.memory[self.memory_index] == 0 {
            let mut brackets = 0;
            while self.program_counter < self.program.len() {
                let command = self.program[self.program_counter];
                match command {
                    '[' => {
                        brackets += 1;
                    }
                    ']' => {
                        brackets -= 1;
                    }
                    _ => {}
                }

                self.program_counter += 1;

                if brackets == 0 {
                    self.program_counter_updated = true;
                    break;
                }
            }
        } else {
            self.loops.push(self.program_counter);
        }
    }

    fn loop_end(&mut self) {
        if let Some(program_counter) = self.loops.pop() {
            self.program_counter = program_counter;
            self.program_counter_updated = true;
        }
    }

    fn read(&mut self) {
        let mut buffer = [0; 1];
        if self.input.read(&mut buffer).is_ok() {
            self.maybe_grow_memory();
            self.memory[self.memory_index] = buffer[0];
        }
    }

    fn write(&mut self) {
        self.maybe_grow_memory();
        let value = self.memory[self.memory_index];
        self.output_buffer.push(value);

        if value == b'\n' {
            self.flush_output_buffer();
        }
    }

    fn flush_output_buffer(&mut self) {
        self.output.write(&self.output_buffer[..]).unwrap();
        self.output.flush().unwrap();
        self.output_buffer.clear();
    }

    fn maybe_grow_memory(&mut self) {
        let new_len = self.memory_index + 1;
        if new_len > self.memory.len() {
            self.memory.resize(new_len, 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, BufWriter};
    use super::*;

    #[test]
    fn simple() {
        let empty_vec = || Vec::new();
        let empty_string = || "".to_string();

        assert_eq!(empty_vec(), run("", "").1);
        assert_eq!([2].to_vec(), run("++", "").1);
        assert_eq!([2, 1].to_vec(), run("++>+", "").1);
        assert_eq!([2, 1, 0, 1].to_vec(), run("++>+>>+", "").1);
        assert_eq!((empty_string(), empty_vec(), 0), run("", ""));
        assert_eq!((empty_string(), [2].to_vec(), 0), run("++", ""));
        assert_eq!((empty_string(), empty_vec(), 0),
                   run(repeat_char(b'+', 256).as_str(), ""));
        assert_eq!((empty_string(), [2].to_vec(), 0),
                   run("+ random text should be ignored +", ""));
        assert_eq!((empty_string(), [2, 254].to_vec(), 1), run("++>--", ""));
        assert_eq!(("a".to_string(), [97].to_vec(), 0),
                   run((repeat_char(b'+', 'a' as usize) + ".").as_str(), ""));
        assert_eq!((empty_string(), [98].to_vec(), 0), run(",+", "a"));
        assert_eq!(("b".to_string(), [98].to_vec(), 0), run(",+.", "a"));
        assert_eq!((empty_string(), empty_vec(), 0), run("[]", ""));
        assert_eq!((empty_string(), empty_vec(), 0), run("[++]", ""));
        assert_eq!((empty_string(), empty_vec(), 0), run("+[-]", ""));
        assert_eq!((empty_string(), [1].to_vec(), 0), run("++[-]+", ""));
        assert_eq!((empty_string(), empty_vec(), 0), run("++[-]", ""));
        assert_eq!((empty_string(), empty_vec(), 0), run("++[-[-]]", ""));
        assert_eq!((empty_string(), empty_vec(), 0), run("++[[-]+[-]]", ""));
        assert_eq!((empty_string(), [2].to_vec(), 1), run("+++[->[-<->]]", ""));
        assert_eq!((empty_string(), empty_vec(), 1), run("++>++[-<->]", ""));
        assert_eq!(("3".to_string(), [0, 51].to_vec(), 1),
                   run("++>,<[->-<]>.", "5"));
        assert_eq!((empty_string(), [0, 0, 2].to_vec(), 0),
                   run("+++[+[-]+>>++[+---]++<<++[-]]", ""));
        assert_eq!(("H".to_string(), [0, 72, 100, 30, 10].to_vec(), 1),
                   run("++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.", ""));
        assert_eq!(("Hello World!\n".to_string(), [0, 87, 100, 33, 10].to_vec(), 4),
                   run("++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.
                       >+.+++++++..+++.>++.<<+++++++++++++++.>.+++.---
                       ---.--------.>+.>.",
                       ""));
    }

    fn run<S: Into<String>>(program: S, input: S) -> (String, Vec<u8>, usize) {
        let program = program.into();

        let input = input.into();
        let input = input.as_bytes();
        let input = BufReader::new(input);

        let output: Vec<u8> = vec![];
        let output = BufWriter::new(output);

        let mut vm = VM::new(program, input, output);
        vm.run();

        let output_string = vm.get_output_mut()
            .get_mut()
            .by_ref()
            .as_slice()
            .iter()
            .map(|x| *x as char)
            .collect::<String>();

        (output_string, vm.filtered_memory(), vm.memory_index)
    }

    fn repeat_char(ch: u8, n: usize) -> String {
        String::from_utf8(vec![ch; n]).unwrap()
    }
}
