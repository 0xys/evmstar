use ethereum_types::{
    Address, U256, U512
};
use core::convert::TryInto;
use num::traits::FromPrimitive;
use bytes::Bytes;

use crate::model::{
    opcode::OpCode,
    code::{
        CodeError
    },
    revision::Revision,
    evmc::StatusCode
};
use crate::executor::{
    callstack::CallContext,
};
use crate::resume::{
    Resume,
};
use crate::interpreter::{
    stack::{Stack, StackOperationError, Memory},
    Interrupt,
    utils::{
        exp,
        memory::{mload, mstore, mstore8}
    },
};
use crate::utils::{
    u256_to_address,
    i256::{I256, Sign},
};

#[derive(Clone, Debug)]
pub struct Interpreter {
    pub pc: usize,
    pub stack: Stack,
    pub revision: Revision,
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter {
            pc: 0,
            stack: Stack::default(),
            revision: Revision::Shanghai
        }
    }
}

impl Interpreter {
    pub fn resume_interpret(&self, resume: Resume, context: &mut CallContext) -> Result<Interrupt, InterpreterError> {
        self.apply_resume(resume, &mut context.stack, &mut context.memory);
        
        let gas_left = i32::max_value();    // TODO

        loop {
            // code must stop at STOP, RETURN
            let byte = context.code.try_get(context.pc).map_err(|e| InterpreterError::CodeError(e))?;            
            if let Some(opcode) = OpCode::from_u8(byte) {

                // handle PUSH instruction
                if let Some(push_num) = opcode.is_push() {
                    let value = U256::from_big_endian(context.code.slice(context.pc+1,push_num));
                    context.stack.push(value);
                    context.pc += 1 + push_num;
                    continue;
                }
                
                // handle CODECOPY instruction
                if opcode == OpCode::CODECOPY {
                    let dest_offset = context.stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                    let offset = context.stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                    let size = context.stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;

                    let dest_offset = dest_offset.as_usize();
                    let offset = offset.as_usize();
                    let size = size.as_usize();
                    context.memory.set_range(dest_offset, &context.code.0[offset..offset+size]);
                    context.pc += 1;
                    continue;
                }

                match self.next_instruction(&opcode, &mut context.stack, &mut context.memory, &mut gas_left)? {
                    None => (),
                    Some(i) => {
                        context.pc += 1;
                        return Ok(i)
                    }
                };
            }
            
            context.pc += 1;
        }
    }

    /// resume interpretation with returned value.
    fn apply_resume(&self, resume: Resume, stack: &mut Stack, memory: &mut Memory) {
        match resume {
            Resume::Init => (),
            Resume::Balance(balance) => {
                stack.push(balance)
            },
            _ => {}
        }
    }

    /// interpret next instruction, returning interrupt if needed.
    fn next_instruction(&self, opcode: &OpCode, stack: &mut Stack, memory: &mut Memory, gas_left: &mut i32) -> Result<Option<Interrupt>, InterpreterError> {
        match opcode {
            OpCode::STOP => {
                Ok(Some(Interrupt::Return(Bytes::default())))
            },
            OpCode::ADD => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let ans = a + b;
                stack.push(ans);
                Ok(None)
            },
            OpCode::MUL => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let ans = a * b;
                stack.push(ans);
                Ok(None)
            },
            OpCode::SUB => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let ans = a - b;
                stack.push(ans);
                Ok(None)
            },
            OpCode::DIV => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                if b.is_zero() {
                    stack.push(U256::zero());
                    return Ok(None);
                }
                let ans = a / b;
                stack.push(ans);
                Ok(None)
            },
            OpCode::SDIV => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;

                let a = I256::from(a);
                let b = I256::from(b);

                let ans = a / b;
                stack.push(ans.into());
                Ok(None)
            },
            OpCode::MOD => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                if b.is_zero() {
                    stack.push(U256::zero());
                    return Ok(None);
                }
                let ans = a % b;
                stack.push(ans);
                Ok(None)
            },
            OpCode::SMOD => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;

                if b.is_zero() {
                    stack.push(U256::zero());
                    return Ok(None);
                }
                let a = I256::from(a);
                let b = I256::from(b);

                let ans = a % b;
                stack.push(ans.into());
                Ok(None)
            },
            OpCode::ADDMOD => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let m = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;

                let a = U512::from(a);
                let b = U512::from(b);
                let m = U512::from(m);

                let r = if m.is_zero() {
                    U256::zero()
                }else{
                    let r = (a + b) % m;
                    r.try_into().unwrap()
                };
                stack.push(r);
                Ok(None)
            },
            OpCode::MULMOD => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let m = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;

                let a = U512::from(a);
                let b = U512::from(b);
                let m = U512::from(m);

                let r = if m.is_zero() {
                    U256::zero()
                }else{
                    let r = (a * b) % m;
                    r.try_into().unwrap()
                };
                stack.push(r);
                Ok(None)
            },
            OpCode::EXP => {
                let mut base = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let mut power = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;

                let result = exp(&mut base, &mut power, i64::max_value(), Revision::Shanghai)
                    .map_err(|e| InterpreterError::EvmError(e))?;
                
                stack.push(result.0);
                Ok(None)
            },
            OpCode::SIGNEXTEND => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;

                let v = if a < U256::from(32) {
                    let bit_index = (8 * a.low_u32() + 7) as usize;
                    let bit = b.bit(bit_index);
                    let mask = (U256::one() << bit_index) - U256::one();
                    if bit {
                        b | !mask
                    } else {
                        b & mask
                    }
                } else {
                    b
                };

                stack.push(v);
                Ok(None)
            },

            OpCode::LT => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                stack.push(if a.lt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::GT => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                stack.push(if a.gt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::SLT => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let a = I256::from(a);
                let b = I256::from(b);

                stack.push(if a.lt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::SGT => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let a = I256::from(a);
                let b = I256::from(b);

                stack.push(if a.gt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::EQ => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                stack.push(if a.eq(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::ISZERO => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                stack.push(if a.is_zero() { U256::one()} else { U256::zero() });
                Ok(None)
            },

            OpCode::AND => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                stack.push(a & b);
                Ok(None)
            },
            OpCode::OR => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                stack.push(a | b);
                Ok(None)
            },
            OpCode::XOR => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                stack.push(a ^ b);
                Ok(None)
            },
            OpCode::NOT => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                stack.push(!a);
                Ok(None)
            },
            OpCode::BYTE => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;

                let mut ret = U256::zero();

                for i in 0..256 {
                    if i < 8 && a < 32.into() {
                        let o: usize = a.as_usize();
                        let t = 255 - (7 - i + 8 * o);
                        let bit_mask = U256::one() << t;
                        let value = (b & bit_mask) >> t;
                        ret = ret.overflowing_add(value << i).0;
                    }
                }

                stack.push(ret);
                Ok(None)
            },
            OpCode::SHL => {
                let shift = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let value = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;

                let ret = if value.is_zero() || shift >= U256::from(256) {
                    U256::zero()
                } else {
                    value << shift.as_usize()
                };

                stack.push(ret);
                Ok(None)
            },
            OpCode::SHR => {
                let shift = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let value = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;

                let ret = if value.is_zero() || shift >= U256::from(256) {
                    U256::zero()
                } else {
                    value >> shift.as_usize()
                };

                stack.push(ret);
                Ok(None)
            },
            OpCode::SAR => {
                let shift = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let value = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let value = I256::from(value);

                let ret = if value == I256::zero() || shift >= U256::from(256) {
                    match value.0 {
                        // if value >= 0, pushing 0
                        Sign::Plus | Sign::Zero => U256::zero(),
                        // if value < 0, pushing -1
                        Sign::Minus => I256(Sign::Minus, U256::one()).into(),
                    }
                } else {
                    let shift = shift.as_usize();
            
                    match value.0 {
                        Sign::Plus | Sign::Zero => value.1 >> shift,
                        Sign::Minus => {
                            let shifted = ((value.1.overflowing_sub(U256::one()).0) >> shift)
                                .overflowing_add(U256::one())
                                .0;
                            I256(Sign::Minus, shifted).into()
                        }
                    }
                };
            
                stack.push(ret);
                Ok(None)
            },


            OpCode::BALANCE => {
                let address = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let address = u256_to_address(address);
                Ok(Some(Interrupt::Balance(address)))
            },

            OpCode::POP => {
                stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                Ok(None)
            },
            OpCode::MLOAD => {
                let offset = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let gas_consumed = mload(offset, memory, stack, *gas_left).map_err(|e| InterpreterError::EvmError(e))?;
                *gas_left -= gas_consumed;
                Ok(None)
            },
            OpCode::MSTORE => {
                let offset = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let gas_consumed = mstore(offset, memory, stack, *gas_left).map_err(|e| InterpreterError::EvmError(e))?;
                *gas_left -= gas_consumed;
                Ok(None)
            },
            OpCode::MSTORE8 => {
                let offset = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let gas_consumed = mstore8(offset, memory, stack, *gas_left).map_err(|e| InterpreterError::EvmError(e))?;
                *gas_left -= gas_consumed;
                Ok(None)
            }


            OpCode::RETURN => {
                let offset = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let size = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let a = memory.get_range(offset.low_u32() as usize, size.low_u32() as usize);
                Ok(Some(Interrupt::Return(Bytes::from(a.to_owned()))))
            }

            _ => Ok(None)
        }
    }
}

#[derive(Clone, Debug)]
pub enum InterpreterError {
    StackOperationError(StackOperationError),
    CodeError(CodeError),
    EvmError(StatusCode)
}