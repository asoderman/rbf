use std::collections::HashMap;

lazy_static!{
    static ref SYMBOL_MAP: HashMap<char, &'static str> = hashmap! {
            '>' => "    inc    r8\n",
            '<' => "    dec    r8\n",
            '+' => "    inc    byte [r8]\n",
            '-' => "    dec    byte [r8]\n",
            '.' => "    mov    r10b, byte [r8]\n    mov    byte [r9], r10b\n    inc    r9\n",
            ',' => "\n",
            '[' => "\n.loop{}:\n",
            ']' => "    cmp     byte [r8], 0\n    jnz    .loop{}\n"
    };
}

#[derive(Default)]
pub struct Compilation  {
    input: String,
}

impl Compilation {
    pub fn new(input: String) -> Compilation {
        Compilation {
            input,
            ..Default::default()
        }
    }

    // FIXME: This and output_calls() can both be done in 1 iteration 
    // and should most likely be cached within the struct
    fn cells(&self) -> usize {
        self.input.matches('>').count() + 1
    }

    // FIXME: see cells()
    fn output_calls(&self) -> usize {
        self.input.matches('.').count()
    }

    pub fn generate_assembly(&self) -> String {
        let mut output = String::new();
        // Header
        output.push_str("global    _main\n");
        output.push_str("extern    _printf\n");
        output.push_str("default    rel\n");
        output.push('\n');
        output.push('\n');
        output.push_str("section   .text\n");
        output.push_str("_main:  \n");
        output.push_str("    sub    rsp, 8 ; Align the stack pointer \n");
        output.push_str("    mov    r8, cell ; r8 will be our data pointer\n");
        output.push_str("    mov    r9, output ; r9 will be our output pointer\n");

        let mut loop_counter = 0;
        let mut loops_open = Vec::new();
        for c in self.input.chars() {
            if let Some(instruction) = SYMBOL_MAP.get(&c) {
                let i = if c == '[' {
                    loop_counter += 1;
                    loops_open.push(loop_counter);
                    instruction.replace("{}", &loop_counter.to_string())
                } else if c == ']' {
                    let l = loops_open.pop().unwrap();
                    instruction.replace("{}", &l.to_string())
                } else {
                    instruction.to_string()
                };

                output.push_str(&i);
            }
        }

        output.push_str("    mov rax, 0\n");
        output.push_str("    mov rdi, output\n");
        output.push_str("    call _printf\n");
        output.push_str("    add rsp, 8 ; Undo stack alignment\n");
        output.push_str("ret\n");
        output.push_str("    mov    rax, 0x2000001 ; Exit\n");
        output.push_str("    mov    rdi, 0\n");
        output.push_str("    syscall\n\n");

        // bss
        output.push_str("section   .bss\n");
        output.push('\n');
        output.push_str(&format!("output: resb {} \n", self.output_calls()));
        output.push_str(&format!("cell:   resb {} \n", self.cells()));

        output
    }
}


