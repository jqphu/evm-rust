use crate::instruction::Instruction;
use crate::Transaction;
use bytes::Bytes;

#[derive(Debug)]
pub enum Error {}

/// EVM Implementation
#[derive(Debug)]
pub struct Vm {
    /// The code we're executing.
    code: Bytes,

    /// The program counter into code in bytes.
    pc: usize,
}

impl Default for Vm {
    fn default() -> Self {
        Vm::new()
    }
}

impl Vm {
    pub fn new() -> Vm {
        Vm {
            code: Bytes::new(),
            pc: 0,
        }
    }

    pub fn exec(mut self, transaction: Transaction) -> Result<(), Error> {
        self.code = transaction.code;
        self.pc = 0;

        if self.code.is_empty() {
            return Ok(());
        }

        loop {
            let instruction = Instruction::try_from(self.code[self.pc])
                .expect("Must be able to unwrap. Valid codes only.");

            match instruction {
                Instruction::Stop => return Ok(()),
                Instruction::Push32 => self.pc += 32,
                Instruction::Push1 => self.pc += 1,
                _ => {}
            }

            self.pc += 1;
        }
    }
}
