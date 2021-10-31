#[derive(Default, Debug)]
struct State {
    data: Vec<u8>,
    ptr: usize,
    in_loop: bool,
    loop_start: usize
}

impl State {
    fn new() -> Self {
        let data = vec![0; 30_000];

        State {
            data,
            ..Self::default()
        }
    }


    fn deref(&self) -> u8 {
        self.data[self.ptr]
    }

    fn increment_ptr(&mut self) {
        self.ptr += 1;
    }

    fn decrement_ptr(&mut self) {
        self.ptr -= 1;
    }

    fn increment_value(&mut self) {
        self.data[self.ptr] += 1;
    }

    fn decrement_value(&mut self) {
        self.data[self.ptr] -= 1;
    }

    fn putchar(&mut self) {
        print!("{}", self.data[self.ptr] as char)
    }

    fn getchar(&mut self) {
        todo!()
    }
}

#[derive(Default)]
pub struct Interpreter {
    input: String,
    index: usize,
    loop_entry: Vec<usize>,
    state: State
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            state: State::new(),
            ..Interpreter::default()
        }
    }
    pub fn execute(&mut self) {
        loop {
            if let Some(symbol) = self.input.chars().nth(self.index) {
                self.parse(symbol);
                self.index += 1;
            } else {
                break;
            }

        }
    }

    fn parse(&mut self, c: char) {
        /*
        println!("{:?}", self.state.data.iter().cloned().take(10).collect::<Vec<u8>>());
        println!("ptr: {}", self.state.ptr);
        println!("index: {}", self.index);
        println!("char: {}", c);
        */
        match c {
            '>' => self.state.increment_ptr(),
            '<' => self.state.decrement_ptr(),
            '+' => self.state.increment_value(),
            '-' => self.state.decrement_value(),
            '.' => self.state.putchar(),
            ',' => self.state.getchar(),
            '[' => {
                if self.state.deref() == 0u8 {
                    for (i, c) in self.input.chars().enumerate().skip(self.index) {
                        if c == ']' {
                            self.index = i;
                            break;
                        }
                    }
                } else {
                    self.loop_entry.push(self.index);
                }
            },
            ']' => {
                if self.state.deref() != 0u8 {
                    self.index = *self.loop_entry.last().unwrap();
                } else {
                    self.loop_entry.pop();
                }
            },
            _ => ()
        }
    }

    pub fn set_input(&mut self, s: String) {
        self.input = s;
    }
}
