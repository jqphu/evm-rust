use num_enum::TryFromPrimitive;

/// Virtual machine instructions.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Debug, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum Instruction {
    /// Stopping instructions.
    Stop = 0x00,
    /// Adding operation
    Add = 0x01,

    /// Save word to storage.
    SStore = 0x55,

    /// Push a single byte on the stack
    Push1 = 0x60,
    /// Push 32 bytes on the stack
    Push32 = 0x7F,
}
