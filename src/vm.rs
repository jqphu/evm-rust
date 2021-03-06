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

    /// CallData bytes.
    data: Bytes,

    /// EVM Stack
    stack: Stack,

    /// Memory
    memory: Vec<u8>,

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
            data: Bytes::new(),
            memory: [0; 1_048_576].to_vec(),
            stack: Stack::new(),
            pc: 0,
        }
    }

    pub fn exec(mut self, transaction: Transaction) -> Result<Option<Vec<u8>>, Error> {
        info!(
            "

        ############################
            Executing transaction
        ############################
            "
        );
        self.code = transaction.code;
        self.data = transaction.data;
        self.pc = 0;

        if self.code.is_empty() {
            info!("No code provided, exiting");
            return Ok(None);
        }

        loop {
            if self.pc >= self.code.len() {
                return Ok(None);
            }

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
                Instruction::Stop => return Ok(None),
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
                    debug!("b: 0x{:x}, x: 0x{:x}", b, x);

                    if b >= U256::from(32) {
                        self.stack.push(x)
                    } else {
                        let original_length_bits = b
                            .overflowing_add(U256::from(1_u8))
                            .0
                            .overflowing_mul(U256::from(8_u8))
                            .0;

                        let is_leading_set = x.bit(original_length_bits.as_usize() - 1);
                        debug!(
                            "original_length_bits {}, is leading set {}",
                            original_length_bits, is_leading_set
                        );

                        let mask = (U256::one() << original_length_bits) - U256::one();

                        debug!("mask: {:?}", mask);
                        self.stack.push(if is_leading_set {
                            // Set everything as 1s.
                            x | !mask
                        } else {
                            // Set everything else to 0s
                            x & mask
                        });
                    }
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

                Instruction::CallDataLoad => {
                    let i = self.stack.pop()?.as_usize();

                    self.stack.push(U256::from(&self.data[i..i + 32]));
                }

                Instruction::CallDataSize => {
                    self.stack.push(U256::from(self.data.len()));
                }

                Instruction::Pop => {
                    self.stack.pop()?;
                }

                Instruction::MLoad => {
                    let offset = self.stack.pop()?.as_usize();

                    self.stack
                        .push(U256::from(&self.memory[offset..offset + 32]));
                }

                Instruction::MStore => {
                    let offset = self.stack.pop()?.as_usize();
                    let value = self.stack.pop()?;

                    value.to_big_endian(&mut self.memory[offset..offset + 32]);
                }

                Instruction::MStore8 => {
                    let offset = self.stack.pop()?.as_usize();
                    let value = self.stack.pop()?;

                    self.memory[offset] = value.low_u32() as u8;
                }

                Instruction::SLoad => {
                    let key = self.stack.pop()?;

                    self.stack
                        .push(self.storage[&H256::from_uint(&key)].into_uint());
                }

                Instruction::SStore => {
                    let key = self.stack.peek(0)?;
                    let value = self.stack.peek(1)?;

                    self.storage
                        .insert(H256::from_uint(&key), H256::from_uint(&value));
                }

                Instruction::Jump => {
                    let destination = self.stack.pop()?;

                    self.pc = destination.as_usize();
                }

                Instruction::JumpI => {
                    let destination = self.stack.pop()?;
                    let condition = self.stack.pop()?;

                    if !condition.is_zero() {
                        self.pc = destination.as_usize();
                    }
                }

                Instruction::PC => {
                    // Remove the additional bump of the PC above.
                    self.stack.push(U256::from(self.pc - 1));
                }

                // No-op
                Instruction::JumpDest => {}
                // TODO(jqphu): macroify all of this.
                Instruction::Push1
                | Instruction::Push2
                | Instruction::Push3
                | Instruction::Push4
                | Instruction::Push5
                | Instruction::Push6
                | Instruction::Push7
                | Instruction::Push8
                | Instruction::Push9
                | Instruction::Push10
                | Instruction::Push11
                | Instruction::Push12
                | Instruction::Push13
                | Instruction::Push14
                | Instruction::Push15
                | Instruction::Push16
                | Instruction::Push17
                | Instruction::Push18
                | Instruction::Push19
                | Instruction::Push20
                | Instruction::Push21
                | Instruction::Push22
                | Instruction::Push23
                | Instruction::Push24
                | Instruction::Push25
                | Instruction::Push26
                | Instruction::Push27
                | Instruction::Push28
                | Instruction::Push29
                | Instruction::Push30
                | Instruction::Push31
                | Instruction::Push32 => {
                    let bytes = instruction.push_bytes().unwrap();

                    let value = self.read_bytes(bytes);
                    self.stack.push(value);

                    self.pc += bytes;
                }

                Instruction::Swap1
                | Instruction::Swap2
                | Instruction::Swap3
                | Instruction::Swap4
                | Instruction::Swap5
                | Instruction::Swap6
                | Instruction::Swap7
                | Instruction::Swap8
                | Instruction::Swap9
                | Instruction::Swap10
                | Instruction::Swap11
                | Instruction::Swap12
                | Instruction::Swap13
                | Instruction::Swap14
                | Instruction::Swap15
                | Instruction::Swap16 => {
                    let position = instruction.swap_position().unwrap();

                    self.stack.inner.swap(0, position);
                }

                Instruction::Dup1
                | Instruction::Dup2
                | Instruction::Dup3
                | Instruction::Dup4
                | Instruction::Dup5
                | Instruction::Dup6
                | Instruction::Dup7
                | Instruction::Dup8
                | Instruction::Dup9
                | Instruction::Dup10
                | Instruction::Dup11
                | Instruction::Dup12
                | Instruction::Dup13
                | Instruction::Dup14
                | Instruction::Dup15
                | Instruction::Dup16 => {
                    let position = instruction.dup_position().unwrap();

                    self.stack.push(self.stack.peek(position).unwrap());
                }

                Instruction::Return => {
                    let offset = self.stack.pop()?.as_usize();
                    let length = self.stack.pop()?.as_usize();

                    return Ok(Some(self.memory[offset..offset + length].to_vec()));
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
