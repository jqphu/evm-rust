use crate::instruction::Instruction;
use crate::Transaction;
use bytes::Bytes;
use ethereum_types::{BigEndianHash, H256, U256};
use std::collections::HashMap;
use std::vec::Vec;

#[derive(Debug)]
pub enum Error {
    /// Stack error, likely popping too much or peeking too much.
    StackError,
}

/// EVM Implementation
#[derive(Debug)]
pub struct Vm<'a> {
    /// The code we're executing.
    code: Bytes,

    /// EVM Stack
    stack: Stack,

    /// Storage referenced by this VM.
    storage: &'a mut HashMap<H256, H256>,

    /// The program counter into code in bytes.
    pc: usize,
}

impl<'a> Vm<'a> {
    pub fn new(storage: &'a mut HashMap<H256, H256>) -> Vm<'a> {
        Vm {
            storage,
            code: Bytes::new(),
            stack: Stack::new(),
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

            self.pc += 1;

            match instruction {
                Instruction::Stop => return Ok(()),
                Instruction::Push32 => {
                    let value = self.read_bytes(32);
                    self.stack.push(value);

                    self.pc += 32;
                }

                Instruction::Push1 => {
                    let value = self.read_bytes(1);
                    self.stack.push(value);

                    self.pc += 1;
                }

                Instruction::Add => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;

                    let (addition, _) = a.overflowing_add(b);

                    self.stack.push(addition);
                }

                Instruction::SStore => {
                    let key = self.stack.peek(0)?;
                    let value = self.stack.peek(1)?;

                    self.storage
                        .insert(H256::from_uint(&key), H256::from_uint(&value));
                }
            }
        }
    }

    /// Read a given number of bytes
    fn read_bytes(&self, bytes: usize) -> U256 {
        assert!(self.pc + bytes <= self.code.len());

        U256::from(&self.code[self.pc..self.pc + bytes])
    }
}

/// EVM Stack used for convenience.
#[derive(Debug)]
struct Stack {
    /// Stack holding variables, function call arguments and return addressed.
    ///
    /// A stack of uint256
    inner: Vec<U256>,
}

impl Default for Stack {
    fn default() -> Self {
        Stack::new()
    }
}

impl Stack {
    pub fn new() -> Self {
        Stack { inner: Vec::new() }
    }

    pub fn push(&mut self, value: U256) {
        self.inner.push(value);
    }

    pub fn pop(&mut self) -> Result<U256, Error> {
        let result = self.inner.pop();
        match result {
            None => Err(Error::StackError),
            Some(result) => Ok(result),
        }
    }

    pub fn peek(&self, offset_from_top: usize) -> Result<U256, Error> {
        if offset_from_top >= self.inner.len() {
            return Err(Error::StackError);
        }

        Ok(self.inner[self.inner.len() - offset_from_top - 1])
    }
}
