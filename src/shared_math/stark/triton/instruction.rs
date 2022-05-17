use super::ord_n::{Ord16, Ord4, Ord4::*};
use crate::shared_math::b_field_element::BFieldElement;
use std::error::Error;
use std::fmt::Display;
use std::str::SplitWhitespace;
use Instruction::*;
use TokenError::*;

type Word = BFieldElement;

/// A Triton VM instruction
///
/// The ISA is defined at:
///
/// https://neptune.builders/core-team/triton-vm/src/branch/master/specification/isa.md
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    // OpStack manipulation
    Pop,
    Push,
    PushArg(Word),
    Pad,
    Dup,
    DupArg(Ord4),
    Swap,
    SwapArg(Ord4),

    // Control flow
    Skiz,
    Call,
    CallArg(Word),
    Return,
    Recurse,
    Assert,
    Halt,

    // Memory access
    ReadMem,
    WriteMem,

    // Auxiliary register instructions
    Xlix,
    ClearAll,
    Squeeze,
    SqueezeArg(Ord16),
    Absorb,
    AbsorbArg(Ord16),
    MerkleLeft,
    MerkleRight,
    CmpDigest,

    // Arithmetic on stack instructions
    Add,
    Mul,
    Inv,
    Split,
    Eq,
    Lt,
    And,
    Xor,
    Reverse,
    Div,
    XxAdd,
    XxMul,
    XInv,
    XbMul,

    // Read/write
    ReadIo,
    WriteIo,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // OpStack manipulation
            Pop => write!(f, "pop"),
            Push => write!(f, "push"),
            Pad => write!(f, "pad"),
            Dup => write!(f, "dup"),
            Swap => write!(f, "swap"),

            // Control flow
            Skiz => write!(f, "skiz"),
            Call => write!(f, "call"),
            Return => write!(f, "return"),
            Recurse => write!(f, "recurse"),
            Assert => write!(f, "assert"),
            Halt => write!(f, "halt"),

            // Memory access
            ReadMem => write!(f, "read_mem"),
            WriteMem => write!(f, "write_mem"),

            // Auxiliary register instructions
            Xlix => write!(f, "xlix"),
            ClearAll => write!(f, "clearall"),
            Squeeze => write!(f, "squeeze"),
            Absorb => write!(f, "absorb"),
            MerkleLeft => write!(f, "merkle_left"),
            MerkleRight => write!(f, "merkle_right"),
            CmpDigest => write!(f, "cmp_digest"),

            // Arithmetic on stack instructions
            Add => write!(f, "add"),
            Mul => write!(f, "mul"),
            Inv => write!(f, "inv"),
            Split => write!(f, "split"),
            Eq => write!(f, "eq"),
            Lt => write!(f, "lt"),
            And => write!(f, "and"),
            Xor => write!(f, "xor"),
            Reverse => write!(f, "reverse"),
            Div => write!(f, "div"),
            XxAdd => write!(f, "xxadd"),
            XxMul => write!(f, "xxmul"),
            XInv => write!(f, "xinv"),
            XbMul => write!(f, "xbmul"),

            // Read/write
            ReadIo => write!(f, "read_io"),
            WriteIo => write!(f, "write_io"),

            PushArg(arg) => {
                let n: u64 = arg.into();
                write!(f, "{}", n)
            }

            DupArg(arg) => {
                let n: usize = arg.into();
                write!(f, "{}", n)
            }

            SwapArg(arg) => {
                let n: usize = arg.into();
                write!(f, "{}", n)
            }

            CallArg(arg) => {
                let n: u64 = arg.into();
                write!(f, "{}", n)
            }

            SqueezeArg(arg) => {
                let n: usize = arg.into();
                write!(f, "{}", n)
            }

            AbsorbArg(arg) => {
                let n: usize = arg.into();
                write!(f, "{}", n)
            }
        }
    }
}

impl Instruction {
    /// Assign a unique positive integer to each `Instruction`.
    pub fn opcode(&self) -> Option<u32> {
        let value = match self {
            // OpStack manipulation
            Pop => 1,
            Push => 2,
            Pad => 3,
            Dup => 4,
            Swap => 5,

            // Control flow
            Skiz => 10,
            Call => 11,
            Return => 12,
            Recurse => 13,
            Assert => 14,
            Halt => 0,

            // Memory access
            ReadMem => 20,
            WriteMem => 21,

            // Auxiliary register instructions
            Xlix => 30,
            ClearAll => 31,
            Squeeze => 32,
            Absorb => 33,
            MerkleLeft => 34,
            MerkleRight => 35,
            CmpDigest => 36,

            // Arithmetic on stack instructions
            Add => 40,
            Mul => 41,
            Inv => 42,
            Split => 43,
            Eq => 44,
            Lt => 45,
            And => 46,
            Xor => 47,
            Reverse => 48,
            Div => 49,

            XxAdd => 50,
            XxMul => 51,
            XInv => 52,
            XbMul => 53,

            // Read/write
            ReadIo => 71,
            WriteIo => 70,

            PushArg(_) => return None,
            DupArg(_) => return None,
            SwapArg(_) => return None,
            CallArg(_) => return None,
            SqueezeArg(_) => return None,
            AbsorbArg(_) => return None,
        };

        Some(value)
    }

    pub fn size(&self) -> usize {
        match self {
            // Double-word instructions (instructions that take arguments)
            Push => 2,
            Dup => 2,
            Swap => 2,
            Call => 2,
            Squeeze => 2,
            Absorb => 2,

            // Single-word instructions
            Pop => 1,
            Pad => 1,
            Skiz => 1,
            Return => 1,
            Recurse => 1,
            Assert => 1,
            Halt => 1,
            ReadMem => 1,
            WriteMem => 1,
            Xlix => 1,
            ClearAll => 1,
            MerkleLeft => 1,
            MerkleRight => 1,
            CmpDigest => 1,
            Add => 1,
            Mul => 1,
            Inv => 1,
            Split => 1,
            Eq => 1,
            Lt => 1,
            And => 1,
            Xor => 1,
            Reverse => 1,
            Div => 1,
            XxAdd => 1,
            XxMul => 1,
            XInv => 1,
            XbMul => 1,
            WriteIo => 1,
            ReadIo => 1,

            // Arguments (already accounted for)
            PushArg(_) => 0,
            DupArg(_) => 0,
            SwapArg(_) => 0,
            CallArg(_) => 0,
            SqueezeArg(_) => 0,
            AbsorbArg(_) => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub instructions: Vec<Instruction>,
}

impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: Print arguments to multi-word instructions nicely after.
        let mut iterator = self.instructions.iter();
        loop {
            let item = iterator.next();
            if item.is_none() {
                return Ok(());
            }

            let item = item.unwrap();

            match item {
                // Print arguments as separate values
                Push => writeln!(f, "{} {}", item, iterator.next().unwrap())?,
                Call => writeln!(f, "{} {}", item, iterator.next().unwrap())?,

                // Print as argument-less pseudo-instruction ("dup1" instead of "dup 1")
                Dup => writeln!(f, "{}{}", item, iterator.next().unwrap())?,
                Swap => writeln!(f, "{}{}", item, iterator.next().unwrap())?,
                Squeeze => writeln!(f, "{}{}", item, iterator.next().unwrap())?,
                Absorb => writeln!(f, "{}{}", item, iterator.next().unwrap())?,

                instr => writeln!(f, "{}", instr)?,
            }
        }
    }
}

#[derive(Debug)]
pub enum TokenError {
    UnexpectedEndOfStream,
    UnknownInstruction(String),
}

impl Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnknownInstruction(s) => write!(f, "UnknownInstruction({})", s),
            UnexpectedEndOfStream => write!(f, "UnexpectedEndOfStream"),
        }
    }
}

impl Error for TokenError {}

pub fn parse(code: &str) -> Result<Program, Box<dyn Error>> {
    let mut tokens = code.split_whitespace();
    let mut instructions = vec![];

    while let Some(token) = tokens.next() {
        let mut instruction = parse_token(token, &mut tokens)?;
        instructions.append(&mut instruction);
    }

    Ok(Program { instructions })
}

fn parse_token(
    token: &str,
    tokens: &mut SplitWhitespace,
) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let instruction = match token {
        // OpStack manipulation
        "pop" => vec![Pop],
        "push" => vec![Push, PushArg(parse_elem(tokens)?)],
        "pad" => vec![Pad],
        "dup1" => vec![Dup, DupArg(N0)],
        "dup2" => vec![Dup, DupArg(N1)],
        "dup3" => vec![Dup, DupArg(N2)],
        "dup4" => vec![Dup, DupArg(N3)],
        "swap1" => vec![Swap, SwapArg(N1)],
        "swap2" => vec![Swap, SwapArg(N2)],
        "swap3" => vec![Swap, SwapArg(N3)],
        //"swap4" => vec![Swap, SwapArg(N4)],

        // Control flow
        "skiz" => vec![Skiz],
        "call" => vec![Call, CallArg(parse_elem(tokens)?)],
        "return" => vec![Return],
        "recurse" => vec![Recurse],
        "assert" => vec![Assert],
        "halt" => vec![Halt],

        // Memory access
        "read_mem" => vec![ReadMem],
        "write_mem" => vec![WriteMem],

        // Auxiliary register instructions
        "xlix" => vec![Xlix],
        "clearall" => vec![ClearAll],
        "squeeze" => vec![Squeeze, SqueezeArg(parse_arg(tokens)?)],
        "absorb" => vec![Absorb, AbsorbArg(parse_arg(tokens)?)],
        "merkle_left" => vec![MerkleLeft],
        "merkle_right" => vec![MerkleRight],
        "cmp_digest" => vec![CmpDigest],

        // Arithmetic on stack instructions
        "add" => vec![Add],
        "mul" => vec![Mul],
        "inv" => vec![Inv],
        "split" => vec![Split],
        "eq" => vec![Eq],
        "lt" => vec![Lt],
        "and" => vec![And],
        "xor" => vec![Xor],
        "reverse" => vec![Reverse],
        "div" => vec![Div],
        "xxadd" => vec![XxAdd],
        "xxmul" => vec![XbMul],
        "xinv" => vec![XInv],
        "xbmul" => vec![XbMul],

        // Read/write
        "read_io" => vec![ReadIo],
        "write_io" => vec![WriteIo],

        _ => return Err(Box::new(UnknownInstruction(token.to_string()))),
    };

    Ok(instruction)
}

fn parse_arg(tokens: &mut SplitWhitespace) -> Result<Ord16, Box<dyn Error>> {
    let constant_s = tokens.next().ok_or(UnexpectedEndOfStream)?;
    let constant_n = constant_s.parse::<usize>()?;
    let constant_arg = constant_n.try_into()?;

    Ok(constant_arg)
}

fn parse_elem(tokens: &mut SplitWhitespace) -> Result<BFieldElement, Box<dyn Error>> {
    let constant_s = tokens.next().ok_or(UnexpectedEndOfStream)?;

    let mut constant_n128: i128 = constant_s.parse::<i128>()?;
    if constant_n128 < 0 {
        constant_n128 += BFieldElement::QUOTIENT as i128;
    }
    let constant_n64: u64 = constant_n128.try_into()?;
    let constant_elem = BFieldElement::new(constant_n64);

    Ok(constant_elem)
}
pub mod sample_programs {
    use super::{Instruction::*, Program};

    pub const PUSH_PUSH_ADD_POP_S: &str = "
        push 1
        push 2
        add
        pop
    ";

    pub fn push_push_add_pop_p() -> Program {
        let instructions = vec![Push, PushArg(1.into()), Push, PushArg(2.into()), Add, Pop];
        Program { instructions }
    }

    pub const HELLO_WORLD_1: &str = "
        push 10
        push 33
        push 100
        push 108
        push 114
        push 111
        push 87
        push 32
        push 44
        push 111
        push 108
        push 108
        push 101
        push 72

        print print print print print print print print print print print print print print
        ";

    pub const SUBTRACT: &str = "
        push 5
        push 18446744069414584320
        add
    ";

    pub const COUNTDOWN_FROM_10: &str = "
        push 10
        call 4
        push 18446744069414584320
        add
        dup1
        skiz
        recurse
        halt
    ";

    // leave the stack with the n first fibonacci numbers.  f_0 = 0; f_1 = 1
    // buttom-up approach
    pub const FIBONACCI_SOURCE: &str = "
    push 0
    push 1
    push n=6
    -- case: n==0 || n== 1
    dup0
    dup0
    dup0
    mul
    eq
    skiz
    call $basecase
    -- case: n>1
    call $nextline
    call $fib
    swap1 - n on top
    push 18446744069414584320
    add
    skiz
    recurse
    call $basecase
    dup1     :basecase
    push 1
    eq
    skiz
    pop
    pop - remove 1      :endone
    halt
";

    pub const FIBONACCI_int: &str = "
    push 0
    push 1
    push 7
    dup0
    dup0
    dup0
    mul
    eq
    skiz
    call $basecase
    call $nextline
    call $fib
    swap1 - n on top
    push 18446744069414584320
    add
    skiz
    recurse
    call $
    dup1     :basecase
    push 1
    eq
    skiz
    pop
    pop - remove 1      :endone
    halt
    dup3            :fib
    dup3
    add
    return
";

    pub const FIBONACCI: &str = "
    push 0
    push 1
    push 7
    dup1
    dup1
    dup1
    mul
    eq
    skiz
    call 32
    call 19
    call 41
    swap1
    push 18446744069414584320
    add
    dup1
    skiz
    recurse
    call 37
    dup1
    push 0
    eq
    skiz
    pop
    pop
    halt
    dup3
    dup3
    add
    return
";
}

#[cfg(test)]
mod instruction_tests {
    use super::parse;
    use super::sample_programs;

    #[test]
    fn parse_display_push_pop_test() {
        let pgm_expected = sample_programs::push_push_add_pop_p();
        let pgm_pretty = format!("{}", pgm_expected);
        let pgm_actual = parse(&pgm_pretty).unwrap();

        println!("Expected:\n{}", pgm_expected);
        println!("Actual:\n{}", pgm_actual);

        assert_eq!(pgm_expected, pgm_actual);

        let pgm_text = sample_programs::PUSH_PUSH_ADD_POP_S;
        let pgm_actual_2 = parse(pgm_text).unwrap();

        assert_eq!(pgm_expected, pgm_actual_2);
    }
}
