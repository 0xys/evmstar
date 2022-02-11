use ethereum_types::{
    U256, U512
};
use core::convert::TryInto;
use bytes::Bytes;

use crate::model::{
    opcode::OpCode,
    revision::Revision,
    evmc::{
        FailureKind,
        TxContext
    }
};
use crate::executor::{
    callstack::CallContext,
};
use crate::interpreter::{
    stack::{Stack, Memory},
    Interrupt, Resume, ContextKind,
    utils::{
        exp,
        memory::{mload, mstore, mstore8, ret}
    },
};
use crate::utils::{
    i256::{I256, Sign},
    address_to_u256,
};

#[derive(Clone, Debug)]
pub struct Interpreter {
    pub pc: usize,
    pub stack: Stack,
    pub revision: Revision,
    pub trace: bool,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            pc: 0,
            stack: Stack::default(),
            revision: Revision::Shanghai,
            trace: false,
        }
    }
}

impl Interpreter {
    pub fn new_with_tracing() -> Self {
        Self {
            pc: 0,
            stack: Stack::default(),
            revision: Revision::Shanghai,
            trace: true,
        }
    }

    pub fn resume_interpret(&self, resume: Resume, context: &mut CallContext, gas_left: &mut i64) -> Result<Interrupt, FailureKind> {
        self.apply_resume(resume, &mut context.stack, &mut context.memory)?;
        
        let mut old_gas_left = *gas_left;
        loop {
            // code must stop at STOP, RETURN
            let byte = context.code.try_get(context.pc).map_err(|_| FailureKind::Generic("code must stop at STOP, RETURN".to_owned()))?;
            if self.trace {
                println!("{}, {}", old_gas_left - *gas_left, i64::max_value() - *gas_left);
            }
            old_gas_left = *gas_left;
            if let Some(opcode) = OpCode::from_u8(byte) {
                if self.trace {
                    print!("[{}]: {:?} ", context.pc, opcode);
                }

                // handle PUSH instruction
                if let Some(push_num) = opcode.is_push() {
                    Self::consume_constant_gas(gas_left, 3)?;
                    let value = U256::from_big_endian(context.code.slice(context.pc+1,push_num));
                    context.stack.push(value)?;
                    context.pc += 1 + push_num;
                    continue;
                }
                
                // handle CODECOPY instruction
                if opcode == OpCode::CODECOPY {
                    // let dest_offset = context.stack.pop()?;
                    // let offset = context.stack.pop()?;
                    // let size = context.stack.pop()?;

                    // let dest_offset = dest_offset.as_usize();
                    // let offset = offset.as_usize();
                    // let size = size.as_usize();
                    panic!("codecopy not implemented");
                }

                match self.next_instruction(&opcode, context, gas_left)? {
                    None => (),
                    Some(i) => {
                        if i == Interrupt::Jump {
                            continue;   // jump doesn't need incrementing pc.
                        }else{
                            context.pc += 1;
                            return Ok(i)
                        }
                    }
                };
            }
            
            context.pc += 1;
        }
    }

    /// resume interpretation with returned value.
    #[allow(unused_variables)]
    fn apply_resume(&self, resume: Resume, stack: &mut Stack, memory: &mut Memory) -> Result<(), FailureKind> {
        match resume {
            Resume::Init => (),
            Resume::Balance(balance) => {
                stack.push_unchecked(balance);
            },
            Resume::Context(kind, context) => {
                self.handle_resume_context(kind, &context, stack)?
            },
            _ => {}
        }
        Ok(())
    }

    /// interpret next instruction, returning interrupt if needed.
    fn next_instruction(
        &self,
        opcode: &OpCode,
        context: &mut CallContext,
        gas_left: &mut i64
    ) -> Result<Option<Interrupt>, FailureKind> {
        let stack = &mut context.stack;
        let memory = &mut context.memory;

        match opcode {
            OpCode::STOP => {
                Ok(Some(Interrupt::Return(*gas_left, Bytes::default())))
            },
            OpCode::ADD => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let ans = a.overflowing_add(b);
                stack.push_unchecked(ans.0);
                Ok(None)
            },
            OpCode::MUL => {
                Self::consume_constant_gas(gas_left, 5)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let ans = a.overflowing_mul(b);
                stack.push_unchecked(ans.0);
                Ok(None)
            },
            OpCode::SUB => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let ans = a.overflowing_sub(b);
                stack.push_unchecked(ans.0);
                Ok(None)
            },
            OpCode::DIV => {
                Self::consume_constant_gas(gas_left, 5)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                if b.is_zero() {
                    stack.push_unchecked(U256::zero());
                    return Ok(None);
                }
                let ans = a / b;
                stack.push_unchecked(ans);
                Ok(None)
            },
            OpCode::SDIV => {
                Self::consume_constant_gas(gas_left, 5)?;
                let a = stack.pop()?;
                let b = stack.pop()?;

                let a = I256::from(a);
                let b = I256::from(b);

                let ans = a / b;
                stack.push_unchecked(ans.into());
                Ok(None)
            },
            OpCode::MOD => {
                Self::consume_constant_gas(gas_left, 5)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                if b.is_zero() {
                    stack.push_unchecked(U256::zero());
                    return Ok(None);
                }
                let ans = a % b;
                stack.push_unchecked(ans);
                Ok(None)
            },
            OpCode::SMOD => {
                Self::consume_constant_gas(gas_left, 5)?;
                let a = stack.pop()?;
                let b = stack.pop()?;

                if b.is_zero() {
                    stack.push_unchecked(U256::zero());
                    return Ok(None);
                }
                let a = I256::from(a);
                let b = I256::from(b);

                let ans = a % b;
                stack.push_unchecked(ans.into());
                Ok(None)
            },
            OpCode::ADDMOD => {
                Self::consume_constant_gas(gas_left, 8)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let m = stack.pop()?;

                let a = U512::from(a);
                let b = U512::from(b);
                let m = U512::from(m);

                let r = if m.is_zero() {
                    U256::zero()
                }else{
                    let r = (a + b) % m;
                    r.try_into().unwrap()
                };
                stack.push_unchecked(r);
                Ok(None)
            },
            OpCode::MULMOD => {
                Self::consume_constant_gas(gas_left, 8)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let m = stack.pop()?;

                let a = U512::from(a);
                let b = U512::from(b);
                let m = U512::from(m);

                let r = if m.is_zero() {
                    U256::zero()
                }else{
                    let r = (a * b) % m;
                    r.try_into().unwrap()
                };
                stack.push_unchecked(r);
                Ok(None)
            },
            OpCode::EXP => {
                let mut base = stack.pop()?;
                let mut power = stack.pop()?;

                let (gas_consumed, value) = exp(&mut base, &mut power, i64::max_value(), Revision::Shanghai)?;
                
                stack.push_unchecked(value);
                Self::consume_constant_gas(gas_left, gas_consumed)?;
                Ok(None)
            },
            OpCode::SIGNEXTEND => {
                Self::consume_constant_gas(gas_left, 5)?;
                let a = stack.pop()?;
                let b = stack.pop()?;

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

                stack.push_unchecked(v);
                Ok(None)
            },

            OpCode::LT => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(if a.lt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::GT => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(if a.gt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::SLT => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let a = I256::from(a);
                let b = I256::from(b);

                stack.push_unchecked(if a.lt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::SGT => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let a = I256::from(a);
                let b = I256::from(b);

                stack.push_unchecked(if a.gt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::EQ => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(if a.eq(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::ISZERO => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                stack.push_unchecked(if a.is_zero() { U256::one()} else { U256::zero() });
                Ok(None)
            },

            OpCode::AND => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(a & b);
                Ok(None)
            },
            OpCode::OR => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(a | b);
                Ok(None)
            },
            OpCode::XOR => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(a ^ b);
                Ok(None)
            },
            OpCode::NOT => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                stack.push_unchecked(!a);
                Ok(None)
            },
            OpCode::BYTE => {
                Self::consume_constant_gas(gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;

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

                stack.push_unchecked(ret);
                Ok(None)
            },
            OpCode::SHL => {
                Self::consume_constant_gas(gas_left, 3)?;
                let shift = stack.pop()?;
                let value = stack.pop()?;

                let ret = if value.is_zero() || shift >= U256::from(256) {
                    U256::zero()
                } else {
                    value << shift.as_usize()
                };

                stack.push_unchecked(ret);
                Ok(None)
            },
            OpCode::SHR => {
                Self::consume_constant_gas(gas_left, 3)?;
                let shift = stack.pop()?;
                let value = stack.pop()?;

                let ret = if value.is_zero() || shift >= U256::from(256) {
                    U256::zero()
                } else {
                    value >> shift.as_usize()
                };

                stack.push_unchecked(ret);
                Ok(None)
            },
            OpCode::SAR => {
                Self::consume_constant_gas(gas_left, 3)?;
                let shift = stack.pop()?;
                let value = stack.pop()?;
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
            
                stack.push_unchecked(ret);
                Ok(None)
            },

            OpCode::ORIGIN => {
                Self::consume_constant_gas(gas_left, 2)?;
                let origin = address_to_u256(context.origin);
                stack.push(origin)?;
                Ok(None)
            },
            OpCode::CALLER => {
                Self::consume_constant_gas(gas_left, 2)?;
                let caller = address_to_u256(context.caller);
                stack.push(caller)?;
                Ok(None)
            },
            OpCode::CALLVALUE => {
                Self::consume_constant_gas(gas_left, 2)?;
                stack.push(context.value)?;
                Ok(None)
            },

            OpCode::GASPRICE => {
                Self::consume_constant_gas(gas_left, 2)?;
                Ok(Some(Interrupt::Context(ContextKind::GasPrice)))
            },

            // OpCode::BLOCKHASH => {
            //     Self::consume_constant_gas(gas_left, 20)?;
            //     let height = stack.pop()?;
            //     Ok(Some(Interrupt::Context(ContextKind::BlockHash(height.as_usize()))))
            // },
            OpCode::COINBASE => {
                Self::consume_constant_gas(gas_left, 2)?;
                Ok(Some(Interrupt::Context(ContextKind::Coinbase)))
            },
            OpCode::TIMESTAMP => {
                Self::consume_constant_gas(gas_left, 2)?;
                Ok(Some(Interrupt::Context(ContextKind::Timestamp)))
            },
            OpCode::NUMBER => {
                Self::consume_constant_gas(gas_left, 2)?;
                Ok(Some(Interrupt::Context(ContextKind::Number)))
            },
            OpCode::DIFFICULTY => {
                Self::consume_constant_gas(gas_left, 2)?;
                Ok(Some(Interrupt::Context(ContextKind::Difficulty)))
            },
            OpCode::GASLIMIT => {
                Self::consume_constant_gas(gas_left, 2)?;
                Ok(Some(Interrupt::Context(ContextKind::GasLimit)))
            },
            OpCode::CHAINID => {
                Self::consume_constant_gas(gas_left, 2)?;
                Ok(Some(Interrupt::Context(ContextKind::ChainId)))
            },
            // OpCode::SELFBALANCE => {
            //     Self::consume_constant_gas(gas_left, 5)?;
            //     Ok(None)
            // },
            OpCode::BASEFEE => {
                Self::consume_constant_gas(gas_left, 2)?;
                Ok(Some(Interrupt::Context(ContextKind::BaseFee)))
            }

            OpCode::POP => {
                Self::consume_constant_gas(gas_left, 2)?;
                stack.pop()?;
                Ok(None)
            },
            OpCode::MLOAD => {
                Self::consume_constant_gas(gas_left, 3)?;
                let offset = stack.pop()?;
                let gas_consumed = mload(offset, memory, stack, *gas_left).map_err(|e| e)?;
                *gas_left -= gas_consumed;
                Ok(None)
            },
            OpCode::MSTORE => {
                Self::consume_constant_gas(gas_left, 3)?;
                let offset = stack.pop()?;
                let gas_consumed = mstore(offset, memory, stack, *gas_left)?;
                *gas_left -= gas_consumed;
                Ok(None)
            }, 
            OpCode::MSTORE8 => {
                Self::consume_constant_gas(gas_left, 3)?;
                let offset = stack.pop()?;
                let gas_consumed = mstore8(offset, memory, stack, *gas_left)?;
                *gas_left -= gas_consumed;
                Ok(None)
            },
            OpCode::JUMP => {
                Self::consume_constant_gas(gas_left, 8)?;
                let dest = stack.pop()?;
                let dest = dest.as_usize();
                if dest < context.code.0.len() && context.code.0[dest] == OpCode::JUMPDEST.to_u8() {
                    context.pc = dest;
                }else{
                    return Err(FailureKind::BadJumpDestination);
                }
                Ok(Some(Interrupt::Jump))
            },
            OpCode::JUMPI => {
                Self::consume_constant_gas(gas_left, 10)?;
                let dest = stack.pop()?;
                let dest = dest.as_usize();
                let cond = stack.pop()?;
                if cond.is_zero() {
                    if dest < context.code.0.len() && context.code.0[dest] == OpCode::JUMPDEST.to_u8() {
                        context.pc = dest;
                    }else{
                        return Err(FailureKind::BadJumpDestination);
                    }
                }
                Ok(Some(Interrupt::Jump))
            },
            OpCode::PC => {
                Self::consume_constant_gas(gas_left, 2)?;
                let pc = U256::from(context.pc);
                stack.push(pc)?;
                Ok(None)
            }
            OpCode::MSIZE => {
                Self::consume_constant_gas(gas_left, 2)?;
                let len = U256::from(memory.0.len());
                stack.push(len)?;
                Ok(None)
            },
            OpCode::JUMPDEST => {
                Self::consume_constant_gas(gas_left, 1)?;
                Ok(None)
            }
            
            // PUSH instruction is already handled in `resume_interpret()`

            OpCode::DUP1
            | OpCode::DUP2
            | OpCode::DUP3
            | OpCode::DUP4
            | OpCode::DUP5
            | OpCode::DUP6
            | OpCode::DUP7
            | OpCode::DUP8
            | OpCode::DUP9
            | OpCode::DUP10
            | OpCode::DUP11
            | OpCode::DUP12
            | OpCode::DUP13
            | OpCode::DUP14
            | OpCode::DUP15
            | OpCode::DUP16 => {
                Self::consume_constant_gas(gas_left, 3)?;
                let offset = opcode.to_usize() - OpCode::DUP1.to_usize();
                let item = stack.peek_at(offset)?;
                stack.push(item)?;
                Ok(None)
            },

            OpCode::SWAP1
            | OpCode::SWAP2
            | OpCode::SWAP3
            | OpCode::SWAP4
            | OpCode::SWAP5
            | OpCode::SWAP6
            | OpCode::SWAP7
            | OpCode::SWAP8
            | OpCode::SWAP9
            | OpCode::SWAP10
            | OpCode::SWAP11
            | OpCode::SWAP12
            | OpCode::SWAP13
            | OpCode::SWAP14
            | OpCode::SWAP15
            | OpCode::SWAP16 => {
                Self::consume_constant_gas(gas_left, 3)?;
                let offset = opcode.to_usize() - OpCode::SWAP1.to_usize() + 1;
                stack.swap(offset)?;
                Ok(None)
            },

            OpCode::RETURN => {
                let offset = stack.pop()?;
                let size = stack.pop()?;
                let (gas_consumed, data) = ret(offset, size, memory, *gas_left)?;
                *gas_left -= gas_consumed;
                Ok(Some(Interrupt::Return(*gas_left, data)))
            }

            _ => Ok(None)
        }
    }

    fn consume_constant_gas(gas_left: &mut i64, gas: i64) -> Result<(), FailureKind> {
        if *gas_left - gas < 0 {
            return Err(FailureKind::OutOfGas);
        }
        *gas_left -= gas;
        Ok(())
    }

    fn handle_resume_context(&self, kind: ContextKind, context: &TxContext, stack: &mut Stack) -> Result<(), FailureKind> {
        match kind {
            ContextKind::Coinbase => {
                let coinbase = address_to_u256(context.coinbase);
                stack.push(coinbase).map_err(|_| FailureKind::StackOverflow)?;
            },
            ContextKind::Timestamp => {
                let timestamp = U256::from(context.block_timestamp);
                stack.push(timestamp).map_err(|_| FailureKind::StackOverflow)?;
            },
            ContextKind::Number => {
                let number = U256::from(context.block_number);
                stack.push(number).map_err(|_| FailureKind::StackOverflow)?;
            },
            ContextKind::Difficulty => {
                stack.push(context.difficulty).map_err(|_| FailureKind::StackOverflow)?;
            },
            ContextKind::GasLimit => {
                let gas_limit = U256::from(context.gas_limit);
                stack.push(gas_limit).map_err(|_| FailureKind::StackOverflow)?;
            },
            ContextKind::GasPrice => {
                let gas_price = U256::from(context.gas_price);
                stack.push(gas_price).map_err(|_| FailureKind::StackOverflow)?;
            },
            ContextKind::ChainId => {
                stack.push(context.chain_id).map_err(|_| FailureKind::StackOverflow)?;
            },
            ContextKind::BaseFee => {
                stack.push(context.base_fee).map_err(|_| FailureKind::StackOverflow)?;
            }
        };

        Ok(())
    }
}