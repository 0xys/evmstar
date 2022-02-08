use ethereum_types::{
    Address, U256, U512
};
use core::convert::TryInto;
use num::traits::FromPrimitive;

use crate::model::{
    opcode::OpCode,
    code::{
        CodeError
    },
    revision::Revision,
};
use crate::executor::{
    callstack::CallContext,
};
use crate::resume::{
    Resume,
};
use crate::interpreter::{
    stack::{Stack, StackOperationError, Memory},
    Interrupt
};
use crate::utils::{
    u256_to_address,
};

pub struct Interpreter {
    pub pc: usize,
    pub stack: Stack,
    pub revision: Revision,
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
            OpCode::STOP => Ok(Some(Interrupt::Return)),
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

                let ans = a / b;
                stack.push(ans);
                panic!("SDIV not defained");
                // Ok(None)
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
                let ans = a % b;
                stack.push(ans);
                panic!("SMOD not defained");
                // Ok(None)
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
                let base = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let power = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;

                panic!("EXP not defained");
                // Ok(None)
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
                stack.push(if a.lt(&b) { U256::one()} else { U256::zero() });
                panic!("SLT not implemented");
                Ok(None)
            },
            OpCode::SGT => {
                let a = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let b = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                stack.push(if a.gt(&b) { U256::one()} else { U256::zero() });
                panic!("SGT not implemented");
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
                panic!("SAR not implemented.");
                Ok(None)
            }

            OpCode::BALANCE => {
                let address = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let address = u256_to_address(address);
                Ok(Some(Interrupt::Balance(address)))
            }

            _ => Ok(None)
        }
    }
}

pub enum InterpreterError {
    StackOperationError(StackOperationError),
    CodeError(CodeError)
}