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

    /// Equality
    Eq = 0x14,

    /// Save word to storage.
    SStore = 0x55,

    /// Push a single byte on the stack
    Push1 = 0x60,
    /// Push 32 bytes on the stack
    Push32 = 0x7F,
}
