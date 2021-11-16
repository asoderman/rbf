use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedLoopClose(usize)
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::UnexpectedLoopClose(num) => format!("Unexpected loop close at character: {}", num),
        };
        write!(f,"{}", s)
    }
}

pub enum Ops {
    IncrementPtr,
    DecrementPtr,
    Increment,
    Decrement,
    PutChar,
    GetChar,
    OpenLoop,
    CloseLoop
}

pub fn parse(input: &str) -> Result<Vec<Ops>, ParseError> {
    let mut loops = Vec::new();
    let mut loop_counter = 0;

    let mut ops = Vec::new();

    for (i, c) in input.chars().enumerate() {
        let op = match c {
            '>' => Some(Ops::IncrementPtr),
            '<' => Some(Ops::DecrementPtr),
            '+' => Some(Ops::Increment),
            '-' => Some(Ops::Decrement),
            '.' => Some(Ops::PutChar),
            ',' => Some(Ops::GetChar),
            '[' => {
                loops.push(loop_counter);
                loop_counter += 1;
                Some(Ops::OpenLoop)
            },
            ']' => {
                if loops.pop().is_none() {
                    return Err(ParseError::UnexpectedLoopClose(i));
                }
                Some(Ops::CloseLoop)
            },
            _ => None
        };

        op.map(|o| ops.push(o));
    }

    Ok(ops)
}
