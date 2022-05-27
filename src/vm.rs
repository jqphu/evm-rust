use crate::instruction::Instruction;
use crate::Transaction;
use bytes::Bytes;
use ethereum_types::{BigEndianHash, H256, U256};
use log::{debug, error, info, trace};
use std::collections::HashMap;
use std::vec::Vec;

#[derive(Debug)]
pub enum Error {
    /// Invalid Instruction.
    InvalidInstructionError,
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
        info!(
            "

        ############################
            Executing transaction
        ############################
            "
        );
        self.code = transaction.code;
        self.pc = 0;

        if self.code.is_empty() {
            info!("No code provided, exiting");
            return Ok(());
        }

        loop {
            let instruction = Instruction::try_from(self.code[self.pc]).map_err(|err| {
                error!(
                    "Unexpected instruction 0x{:x} err: {:?}",
                    self.code[self.pc], err
                );
                Error::InvalidInstructionError
            })?;

            debug!("{:?}", instruction);
            trace!("Pc: {:?}", self.pc);
            trace!("Storage: {:?}", self.storage);
            trace!("Stack: {:?}", self.stack);

            self.pc += 1;

            match instruction {
                Instruction::Stop => return Ok(()),
                Instruction::Add => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;

                    self.stack.push(a.overflowing_add(b).0);
                }

                Instruction::Mul => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;

                    self.stack.push(a.overflowing_mul(b).0);
                }

                Instruction::Sub => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;

                    self.stack.push(a.overflowing_sub(b).0);
                }

                Instruction::Div => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;

                    self.stack
                        .push(if b.is_zero() { U256::zero() } else { a / b });
                }

                Instruction::SDiv => {
                    let (a_abs, sign_a) = get_and_clear_sign(self.stack.pop()?);
                    let (b_abs, sign_b) = get_and_clear_sign(self.stack.pop()?);

                    // Minimum value representable
                    let min = (U256::one() << 255) - U256::one();

                    // If it's zero, just return zero.
                    self.stack.push(if b_abs.is_zero() {
                        U256::zero()
                    } else if a_abs == min && b_abs == !U256::zero() {
                        min
                    } else {
                        let mut result = a_abs / b_abs;
                        let should_negate = sign_a ^ sign_b;

                        if should_negate {
                            result = twos_complement(result)
                        }

                        result
                    })
                }

                Instruction::Mod => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;

                    self.stack
                        .push(if b.is_zero() { U256::zero() } else { a % b });
                }

                Instruction::SMod => {
                    let (a_abs, signed) = get_and_clear_sign(self.stack.pop()?);
                    let (b_abs, _) = get_and_clear_sign(self.stack.pop()?);

                    self.stack.push(if b_abs.is_zero() {
                        U256::zero()
                    } else {
                        let mut result = a_abs % b_abs;

                        if signed {
                            result = twos_complement(result);
                        }

                        // TODO(jqphu): run a test for signed mod.
                        result
                    })
                }

                Instruction::AddMod => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    let modulus = self.stack.pop()?;

                    self.stack.push(if modulus.is_zero() {
                        U256::zero()
                    } else {
                        // Need to do the modulus separately to be careful of overflowing U256.
                        ((a % modulus) + (b % modulus)) % modulus
                    });
                }

                Instruction::MulMod => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    let modulus = self.stack.pop()?;

                    self.stack.push(if modulus.is_zero() {
                        U256::zero()
                    } else {
                        // Need to do the modulus separately to be careful of overflowing U256.
                        ((a % modulus) * (b % modulus)) % modulus
                    });
                }

                Instruction::Exp => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;

                    self.stack.push(a.overflowing_pow(b).0);
                }
                Instruction::SignExtend => {
                    let b = self.stack.pop()?;
                    let x = self.stack.pop()?;

                    let original_length_bits = b
                        .overflowing_add(U256::from(1_u8))
                        .0
                        .overflowing_mul(U256::from(8_u8))
                        .0;
                    let is_leading_set = b.bit(original_length_bits.as_usize());

                    let mask = (U256::one() << b) - U256::one();
                    self.stack.push(if is_leading_set {
                        // Set everything as 1s.
                        x | !mask
                    } else {
                        // Set everything else to 0s
                        x & mask
                    });
                }

                Instruction::Lt => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;

                    self.stack.push(U256::from((a < b) as u8));
                }

                Instruction::Gt => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;

                    self.stack.push(U256::from((a > b) as u8));
                }

                Instruction::Slt => {
                    let (a_abs, a_sign) = get_and_clear_sign(self.stack.pop()?);
                    let (b_abs, b_sign) = get_and_clear_sign(self.stack.pop()?);

                    self.stack.push(
                        // Both positive
                        if !a_sign && !b_sign {
                            U256::from((a_abs < b_abs) as u8)
                            // A is negative.
                        } else if a_sign && !b_sign {
                            U256::from(true as u8)
                            // A is positive, b is negative
                        } else if !a_sign && b_sign {
                            U256::from(false as u8)
                            // Both negative, so reverse the check.
                        } else {
                            U256::from((b_abs < a_abs) as u8)
                        },
                    );
                }

                Instruction::Sgt => {
                    let (a_abs, a_sign) = get_and_clear_sign(self.stack.pop()?);
                    let (b_abs, b_sign) = get_and_clear_sign(self.stack.pop()?);

                    self.stack.push(
                        // Both positive
                        if !a_sign && !b_sign {
                            U256::from((a_abs > b_abs) as u8)
                            // A is negative.
                        } else if a_sign && !b_sign {
                            U256::from(false as u8)
                            // A is positive, b is negative
                        } else if !a_sign && b_sign {
                            U256::from(true as u8)
                            // Both negative, so reverse the check.
                        } else {
                            U256::from((b_abs > a_abs) as u8)
                        },
                    );
                }

                Instruction::Eq => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;

                    self.stack.push(U256::from((a == b) as u8));
                }

                Instruction::IsZero => {
                    let a = self.stack.pop()?;
                    self.stack.push(U256::from(a.is_zero() as u8));
                }

                Instruction::And => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a & b);
                }

                Instruction::Or => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a | b);
                }

                Instruction::Xor => {
                    let a = self.stack.pop()?;
                    let b = self.stack.pop()?;
                    self.stack.push(a ^ b);
                }

                Instruction::Not => {
                    let a = self.stack.pop()?;
                    self.stack.push(!a);
                }

                Instruction::Byte => {
                    let i = self.stack.pop()?;
                    let x = self.stack.pop()?;
                    self.stack.push(U256::from(x.byte(i.as_usize())));
                }

                Instruction::Shl => {
                    let shift = self.stack.pop()?;
                    let value = self.stack.pop()?;
                    // TODO(jqphu): check if overflow already handled.
                    self.stack.push(value << shift.as_usize());
                }

                Instruction::Shr => {
                    let shift = self.stack.pop()?;
                    let value = self.stack.pop()?;
                    self.stack.push(value >> shift.as_usize());
                }

                Instruction::Sar => {
                    let shift = self.stack.pop()?;
                    let (value_abs, value_sign) = get_and_clear_sign(self.stack.pop()?);

                    let shifted_value = value_abs << shift;
                    let top_bit = U256::one() << 255;

                    self.stack.push(if value_sign {
                        shifted_value | top_bit
                    } else {
                        shifted_value
                    });
                }

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

fn get_and_clear_sign(value: U256) -> (U256, bool) {
    let signed = value.bit(255);

    if signed {
        // Negate and add 1 to make it unsigned.
        (twos_complement(value), true)
    } else {
        (value, false)
    }
}

/// Get the two's complement of a number.
fn twos_complement(value: U256) -> U256 {
    (!U256::zero() ^ value).overflowing_add(U256::one()).0
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
