use std::collections::hash_set::HashSet;
use std::num::Wrapping;

type Memory = Vec<u8>;

struct VM {
    program: Vec<char>,
    input: Memory,
    memory: Memory,
    memory_index: usize,
    program_counter: usize,
    output: String,
    output_buffer: String,

    program_counter_updated: bool,
    loops: Vec<usize>,
}

type OutputMemoryIndex = (String, Memory, usize);

const DEFAULT_MEMORY_CAPACITY: usize = 512;
const DEFAULT_OUTPUT_CAPACITY: usize = 512;
const DEFAULT_LOOPS_CAPACITY: usize = 128;

impl VM {
    fn new(program: String, input: Memory) -> VM {
        let supported_commands = hashset!{'+', '-', '<', '>', '[', ']', ',', '.'};
        let filtered_program = program.chars()
            .filter(|x| supported_commands.contains(x))
            .collect::<Vec<char>>();

        VM {
            program: filtered_program,
            input: input,
            memory: Vec::with_capacity(DEFAULT_MEMORY_CAPACITY),
            memory_index: 0,
            program_counter: 0,
            output: String::with_capacity(DEFAULT_OUTPUT_CAPACITY),
            output_buffer: String::with_capacity(DEFAULT_OUTPUT_CAPACITY),

            program_counter_updated: false,
            loops: Vec::with_capacity(DEFAULT_LOOPS_CAPACITY),
        }
    }

    fn run(&mut self) -> OutputMemoryIndex {
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
            print!("{}", self.output_buffer);
            self.output_buffer.clear();
        }

        (self.output.clone(), self.filtered_memory(), self.memory_index)
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
        if let Some(value) = self.input.pop() {
            self.maybe_grow_memory();
            self.memory[self.memory_index] = value;
        }
    }

    fn write(&mut self) {
        self.maybe_grow_memory();
        let value = self.memory[self.memory_index];

        let ch = value as char;
        self.output.push(ch);
        self.output_buffer.push(ch);

        if ch == '\n' {
            print!("{}", self.output_buffer);
            self.output_buffer.clear();
        }
    }

    fn maybe_grow_memory(&mut self) {
        let new_len = self.memory_index + 1;
        if new_len > self.memory.len() {
            self.memory.resize(new_len, 0)
        }
    }

    fn filtered_memory(&self) -> Memory {
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
}

pub fn run<S: Into<String>>(program: S, input: S) -> OutputMemoryIndex {
    let program = program.into();
    let input = input.into()
        .chars()
        .rev()
        .map(|x| x as u8)
        .collect::<Memory>();
    let mut vm = VM::new(program, input);
    vm.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Memory;

    #[test]
    fn simple() {
        let empty_vec = || Memory::new();
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

    fn repeat_char(ch: u8, n: usize) -> String {
        String::from_utf8(vec![ch; n]).unwrap()
    }
}
