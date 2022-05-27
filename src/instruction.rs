use num_enum::TryFromPrimitive;

/// Virtual machine instructions.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum Instruction {
    /// Stopping instructions.
    Stop = 0x00,
    /// Adding operation
    Add = 0x01,
    /// Multiply
    Mul = 0x02,
    /// Subtraction
    Sub = 0x03,
    /// Division
    Div = 0x04,
    /// Signed Division
    SDiv = 0x05,
    /// Modulus
    Mod = 0x06,
    /// Signed Modulus
    SMod = 0x07,
    /// Add then mod.
    AddMod = 0x08,
    /// Multiply then mod.
    MulMod = 0x09,
    /// Exponent.
    Exp = 0x0a,
    /// Extend length of two's complement signed integer.
    SignExtend = 0x0b,

    /// Less than.
    Lt = 0x10,
    /// Greater than.
    Gt = 0x11,
    /// Signed Less than
    Slt = 0x12,
    /// Signed Greater than.
    Sgt = 0x13,
    /// Equality
    Eq = 0x14,
    /// Is Zero
    IsZero = 0x15,
    /// And
    And = 0x16,
    /// Or
    Or = 0x17,
    /// Xor
    Xor = 0x18,
    /// Not
    Not = 0x19,
    /// Byte
    Byte = 0x1a,
    /// Shift left
    Shl = 0x1b,
    /// Shift right
    Shr = 0x1c,
    /// Arithmetic (signed) shift right
    Sar = 0x1d,

    /// Pop from the stack.
    Pop = 0x50,
    /// Load from memory
    MLoad = 0x51,
    /// Store a word to memory
    MStore = 0x52,
    /// Store a byte to memory
    MStore8 = 0x53,
    /// Load from storage
    SLoad = 0x54,
    /// Save word to storage.
    SStore = 0x55,
    /// Jump.
    Jump = 0x56,
    /// Conditional jump
    JumpI = 0x57,
    /// Program counter
    PC = 0x58,

    /// Metadata just to indicate jump destination. No-op.
    JumpDest = 0x5b,

    /// Push n bytes on the stack
    Push1 = 0x60,
    Push2 = 0x61,
    Push3 = 0x62,
    Push4 = 0x63,
    Push5 = 0x64,
    Push6 = 0x65,
    Push7 = 0x66,
    Push8 = 0x67,
    Push9 = 0x68,
    Push10 = 0x69,
    Push11 = 0x6a,
    Push12 = 0x6b,
    Push13 = 0x6c,
    Push14 = 0x6d,
    Push15 = 0x6e,
    Push16 = 0x6f,
    Push17 = 0x70,
    Push18 = 0x71,
    Push19 = 0x72,
    Push20 = 0x73,
    Push21 = 0x74,
    Push22 = 0x75,
    Push23 = 0x76,
    Push24 = 0x77,
    Push25 = 0x78,
    Push26 = 0x79,
    Push27 = 0x7a,
    Push28 = 0x7b,
    Push29 = 0x7c,
    Push30 = 0x7d,
    Push31 = 0x7e,
    Push32 = 0x7f,

    /// Swap 1st and 2nd stack items
    Swap1 = 0x90,
    Swap2 = 0x91,
    Swap3 = 0x92,
    Swap4 = 0x93,
    Swap5 = 0x94,
    Swap6 = 0x95,
    Swap7 = 0x96,
    Swap8 = 0x97,
    Swap9 = 0x98,
    Swap10 = 0x99,
    Swap11 = 0x9a,
    Swap12 = 0x9b,
    Swap13 = 0x9c,
    Swap14 = 0x9d,
    Swap15 = 0x9e,
    Swap16 = 0x9f,

    /// Return
    Return = 0xf3,
}

use Instruction::*;

impl Instruction {
    /// Returns true if given instruction is `PUSHN` instruction.
    pub fn is_push(&self) -> bool {
        *self >= Push1 && *self <= Push32
    }

    /// Returns number of bytes to read for `PUSHN` instruction
    /// Push1 -> 1
    pub fn push_bytes(&self) -> Option<usize> {
        if self.is_push() {
            Some(((*self as u8) - (Push1 as u8) + 1) as usize)
        } else {
            None
        }
    }

    /// Returns stack position of item to SWAP top with
    /// SWAP1 -> 1
    pub fn swap_position(&self) -> Option<usize> {
        if *self >= Swap1 && *self <= Swap16 {
            Some(((*self as u8) - (Swap1 as u8) + 1) as usize)
        } else {
            None
        }
    }
}
