use ethereum_types::{
    U256, U512
};
use std::cmp::min;

use crate::{model::{
    opcode::OpCode,
    revision::Revision,
    evmc::{
        FailureKind,
        TxContext,
        AccessStatus,
    }
}, host::Host};
use crate::executor::{
    callstack::{
        CallScope, ExecutionContext
    }
};
use crate::interpreter::{
    stack::{Stack},
    Interrupt, Resume, ContextKind,
    utils::{
        exp,
        memory::{
            mload, mstore, mstore8, ret, mstore_data, resize_memory,
        },
        gasometer::{calc_sstore_gas_cost, calc_sstore_gas_refund},
    },
};
use crate::utils::{
    i256::{I256, Sign},
    address_to_u256, u256_to_address,
};

use super::{CallParams, CallKind};

#[derive(Clone, Debug)]
pub struct Interpreter {
    pub pc: usize,
    pub stack: Stack,
    pub trace: bool,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            pc: 0,
            stack: Stack::default(),
            trace: false,
        }
    }
}

impl Interpreter {
    pub fn new_with_tracing() -> Self {
        Self {
            pc: 0,
            stack: Stack::default(),
            trace: true,
        }
    }

    pub fn resume_interpret(
        &self,
        resume: Resume,
        scope: &mut CallScope,
        exec_context: &mut ExecutionContext,
        host: &mut Box<dyn Host>
    ) -> Result<Interrupt, FailureKind> {
        let mut old_gas_left = scope.gas_left;
        
        self.apply_resume(resume, scope, exec_context)?;
        
        loop {
            if self.trace {
                let gas_consumed_by_current = old_gas_left - scope.gas_left;
                let gas_consumed_sofar = scope.gas_limit - scope.gas_left;
                println!("{}, {}", gas_consumed_by_current, gas_consumed_sofar);
            }
            old_gas_left = scope.gas_left;

            let op_byte = match scope.code.0.get(scope.pc) {
                Some(num) => *num,
                None => return Ok(Interrupt::Stop(scope.gas_left))
            };

            if let Some(opcode) = OpCode::from_u8(op_byte) {
                if self.trace {
                    print!("[{}]: {:?} ", scope.pc, opcode);
                }

                // handle PUSH instruction
                if let Some(push_num) = opcode.is_push() {
                    Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                    let value = U256::from_big_endian(scope.code.slice(scope.pc+1,push_num));
                    scope.stack.push(value)?;
                    scope.pc += 1 + push_num;
                    continue;
                }
                
                match self.next_instruction(&opcode, scope, exec_context, host)? {
                    None => (),
                    Some(i) => {
                        if i == Interrupt::Jump {
                            continue;   // jump doesn't need incrementing pc.
                        }else{
                            scope.pc += 1;
                            return Ok(i)
                        }
                    }
                };
            }
            
            scope.pc += 1;
        }
    }

    /// resume interpretation with returned value.
    #[allow(unused_variables)]
    fn apply_resume(
        &self,
        resume: Resume,
        call_context: &mut CallScope,
        exec_context: &mut ExecutionContext
    ) -> Result<(), FailureKind> {
        let stack = &mut call_context.stack;
        let memory = &mut call_context.memory;
        
        match resume {
            Resume::Init => (),
            Resume::Returned(success) => {
                stack.push_unchecked(if success { U256::one() } else { U256::zero() });
            },
            _ => {}
        }

        if call_context.gas_left < 0 {
            return Err(FailureKind::OutOfGas);
        }

        Ok(())
    }

    /// interpret next instruction, returning interrupt if needed.
    fn next_instruction(
        &self,
        opcode: &OpCode,
        scope: &mut CallScope,
        exec_context: &mut ExecutionContext,
        host: &mut Box<dyn Host>,
    ) -> Result<Option<Interrupt>, FailureKind> {
        let stack = &mut scope.stack;
        let memory = &mut scope.memory;

        match opcode {
            OpCode::STOP => {
                Ok(Some(Interrupt::Stop(scope.gas_left)))
            },
            OpCode::ADD => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let ans = a.overflowing_add(b);
                stack.push_unchecked(ans.0);
                Ok(None)
            },
            OpCode::MUL => {
                Self::consume_constant_gas(&mut scope.gas_left, 5)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let ans = a.overflowing_mul(b);
                stack.push_unchecked(ans.0);
                Ok(None)
            },
            OpCode::SUB => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let ans = a.overflowing_sub(b);
                stack.push_unchecked(ans.0);
                Ok(None)
            },
            OpCode::DIV => {
                Self::consume_constant_gas(&mut scope.gas_left, 5)?;
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
                Self::consume_constant_gas(&mut scope.gas_left, 5)?;
                let a = stack.pop()?;
                let b = stack.pop()?;

                let a = I256::from(a);
                let b = I256::from(b);

                let ans = a / b;
                stack.push_unchecked(ans.into());
                Ok(None)
            },
            OpCode::MOD => {
                Self::consume_constant_gas(&mut scope.gas_left, 5)?;
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
                Self::consume_constant_gas(&mut scope.gas_left, 5)?;
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
                Self::consume_constant_gas(&mut scope.gas_left, 8)?;
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
                Self::consume_constant_gas(&mut scope.gas_left, 8)?;
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

                let (gas_consumed, value) = exp(&mut base, &mut power, i64::max_value(), exec_context.revision)?;
                
                stack.push_unchecked(value);
                Self::consume_constant_gas(&mut scope.gas_left, gas_consumed)?;
                Ok(None)
            },
            OpCode::SIGNEXTEND => {
                Self::consume_constant_gas(&mut scope.gas_left, 5)?;
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
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(if a.lt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::GT => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(if a.gt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::SLT => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let a = I256::from(a);
                let b = I256::from(b);

                stack.push_unchecked(if a.lt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::SGT => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                let a = I256::from(a);
                let b = I256::from(b);

                stack.push_unchecked(if a.gt(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::EQ => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(if a.eq(&b) { U256::one()} else { U256::zero() });
                Ok(None)
            },
            OpCode::ISZERO => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                stack.push_unchecked(if a.is_zero() { U256::one()} else { U256::zero() });
                Ok(None)
            },

            OpCode::AND => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(a & b);
                Ok(None)
            },
            OpCode::OR => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(a | b);
                Ok(None)
            },
            OpCode::XOR => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                let b = stack.pop()?;
                stack.push_unchecked(a ^ b);
                Ok(None)
            },
            OpCode::NOT => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let a = stack.pop()?;
                stack.push_unchecked(!a);
                Ok(None)
            },
            OpCode::BYTE => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
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
                // EIP-145: https://eips.ethereum.org/EIPS/eip-145
                if exec_context.revision < Revision::Constantinople {
                    return Err(FailureKind::InvalidInstruction);
                }
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
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
                // EIP-145: https://eips.ethereum.org/EIPS/eip-145
                if exec_context.revision < Revision::Constantinople {
                    return Err(FailureKind::InvalidInstruction);
                }
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
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
                // EIP-145: https://eips.ethereum.org/EIPS/eip-145
                if exec_context.revision < Revision::Constantinople {
                    return Err(FailureKind::InvalidInstruction);
                }
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
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

            // OpCode::KECCAK256 => {
            //     Ok(None)
            // },
            OpCode::ADDRESS => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let address= address_to_u256(scope.code_address);
                stack.push(address)?;
                Ok(None)
            },
            OpCode::BALANCE => {
                let address = stack.pop()?;
                let address = u256_to_address(address);
                let access_status = if exec_context.revision >= Revision::Berlin {
                    host.access_account(address)
                }else{
                    AccessStatus::Warm
                };
                let balance = host.get_balance(address);
                let gas = 
                if exec_context.revision >= Revision::Berlin {
                    match access_status {
                        AccessStatus::Cold => 2600,
                        AccessStatus::Warm => 100,
                    }
                }else{
                    if exec_context.revision < Revision::Tangerine {
                        20
                    }else if exec_context.revision < Revision::Istanbul {
                        400
                    }else{
                        700
                    }
                };
                stack.push_unchecked(balance);
                Self::consume_constant_gas(&mut scope.gas_left, gas)?;

                Ok(None)
            },
            OpCode::ORIGIN => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let origin = address_to_u256(scope.origin);
                stack.push(origin)?;
                Ok(None)
            },
            OpCode::CALLER => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let caller = address_to_u256(scope.caller);
                stack.push(caller)?;
                Ok(None)
            },
            OpCode::CALLVALUE => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                stack.push(scope.value)?;
                Ok(None)
            },
            OpCode::CALLDATALOAD => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let offset = stack.pop()?;
                let calldata = scope.calldata.get_word(offset.as_usize());
                stack.push_unchecked(calldata);
                Ok(None)
            },
            OpCode::CALLDATASIZE => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let calldata_length = U256::from(scope.calldata.0.len());
                stack.push(calldata_length)?;
                Ok(None)
            },
            OpCode::CALLDATACOPY => {
                let dest_offset = stack.pop()?;
                let offset = stack.pop()?;
                let size = stack.pop()?;
                let data = scope.calldata.get_range(offset.as_usize(), size.as_usize());
                let dynamic_cost = mstore_data(dest_offset, memory, &data, scope.gas_left)?;
                Self::consume_constant_gas(&mut scope.gas_left, 3 + dynamic_cost)?;    // static cost of `3` is added here.
                Ok(None)
            },
            OpCode::CODESIZE => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let size = scope.code.0.len();
                stack.push(U256::from(size))?;
                Ok(None)
            },
            OpCode::CODECOPY => {
                let dest_offset = stack.pop()?;
                let offset = stack.pop()?;
                let size = stack.pop()?;
                let dynamic_cost = mstore_data(dest_offset, memory, &scope.code.get_range(offset.as_usize(), size.as_usize()), scope.gas_left)?;
                Self::consume_constant_gas(&mut scope.gas_left, 3 + dynamic_cost)?;    // static cost of `3` is added here.
                Ok(None)
            },
            OpCode::GASPRICE => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let tx_context = host.get_tx_context();
                self.handle_context(ContextKind::GasPrice, tx_context, stack)?;
                Ok(None)
            },
            OpCode::EXTCODESIZE => {
                let address = stack.pop()?;
                let address = u256_to_address(address);
                let access_status = if exec_context.revision >= Revision::Berlin {
                    host.access_account(address)
                }else{
                    AccessStatus::Warm
                };
                let size = host.get_code_size(address);
                stack.push_unchecked(size);
                let cost =
                    if exec_context.revision >= Revision::Berlin {
                        match access_status {
                            AccessStatus::Warm => 100,
                            AccessStatus::Cold => 2600,
                        }
                    }else{
                        if exec_context.revision >= Revision::Tangerine {
                            700
                        }else{
                            20
                        }
                    };
                Self::consume_constant_gas(&mut scope.gas_left, cost)?;
                Ok(None)
            },
            OpCode::EXTCODECOPY => {
                let address = stack.pop()?;
                let address = u256_to_address(address);
                let dest_offset = stack.pop()?;
                let offset = stack.pop()?;
                let size = stack.pop()?;
                let access_status = if exec_context.revision >= Revision::Berlin {
                    host.access_account(address)
                }else{
                    AccessStatus::Warm
                };
                let code = host.get_code(address, offset.as_usize(), size.as_usize());

                let memory_cost = mstore_data(U256::from(dest_offset), memory, &code, scope.gas_left)?;
                let account_access_cost = 
                    if exec_context.revision >= Revision::Berlin {
                        match access_status {
                            AccessStatus::Warm => 100,
                            AccessStatus::Cold => 2600,
                        }
                    }else{
                        if exec_context.revision >= Revision::Tangerine {
                            700
                        }else{
                            20
                        }
                    };
                Self::consume_constant_gas(&mut scope.gas_left, account_access_cost + memory_cost)?;
                Ok(None)
            },
            OpCode::RETURNDATASIZE => {
                // EIP-211: https://eips.ethereum.org/EIPS/eip-211
                if exec_context.revision < Revision::Byzantium {
                    return Err(FailureKind::InvalidInstruction);
                }
                let size = exec_context.return_data_buffer.len();
                stack.push(U256::from(size))?;
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                Ok(None)
            },
            OpCode::RETURNDATACOPY => {
                // EIP-211: https://eips.ethereum.org/EIPS/eip-211
                if exec_context.revision < Revision::Byzantium {
                    return Err(FailureKind::InvalidInstruction);
                }
                let dest_offset = stack.pop()?;
                let offset = stack.pop()?;
                let offset = offset.as_usize();
                let size = stack.pop()?;
                let size = size.as_usize();
                let data = exec_context.return_data_buffer.to_vec();
                if offset + size > data.len() {
                    return Err(FailureKind::InvalidMemoryAccess);
                }

                let dynamic_cost = mstore_data(dest_offset, memory, &data[offset..offset+size], scope.gas_left)
                    .map_err(|_| FailureKind::OutOfGas)?;
                Self::consume_constant_gas(&mut scope.gas_left, 3 + dynamic_cost)?;    // static cost of `3` is added here.
                Ok(None)
            },
            OpCode::EXTCODEHASH => {
                // EIP-1052: https://eips.ethereum.org/EIPS/eip-1052
                if exec_context.revision < Revision::Constantinople {
                    return Err(FailureKind::InvalidInstruction);
                }
                let address = stack.pop()?;
                let address = u256_to_address(address);

                let access_status = host.access_account(address);
                let hash = host.get_code_hash(address);

                let gas =
                    if exec_context.revision >= Revision::Berlin {
                        match access_status {
                            AccessStatus::Warm => 100,
                            AccessStatus::Cold => 2600,
                        }
                    }else{
                        match exec_context.revision {
                            Revision::Constantinople | Revision::Petersburg => 400,
                            Revision::Istanbul => 700,
                            _ => {
                                return Err(FailureKind::InvalidInstruction);
                            }
                        }
                    };                
                Self::consume_constant_gas(&mut scope.gas_left, gas)?;
                stack.push_unchecked(hash);
                Ok(None)
            },
            OpCode::BLOCKHASH => {
                Self::consume_constant_gas(&mut scope.gas_left, 20)?;
                let height = stack.pop()?;
                let hash = host.get_blockhash(height.as_usize());
                stack.push_unchecked(hash);
                Ok(None)
            },
            OpCode::COINBASE => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let tx_context = host.get_tx_context();
                self.handle_context(ContextKind::Coinbase, tx_context, stack)?;
                Ok(None)
            },
            OpCode::TIMESTAMP => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let tx_context = host.get_tx_context();
                self.handle_context(ContextKind::Timestamp, tx_context, stack)?;
                Ok(None)
            },
            OpCode::NUMBER => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let tx_context = host.get_tx_context();
                self.handle_context(ContextKind::Number, tx_context, stack)?;
                Ok(None)
            },
            OpCode::DIFFICULTY => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let tx_context = host.get_tx_context();
                self.handle_context(ContextKind::Difficulty, tx_context, stack)?;
                Ok(None)
            },
            OpCode::GASLIMIT => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let tx_context = host.get_tx_context();
                self.handle_context(ContextKind::GasLimit, tx_context, stack)?;
                Ok(None)
            },
            OpCode::CHAINID => {
                // EIP-1344: https://eips.ethereum.org/EIPS/eip-1344
                if exec_context.revision < Revision::Istanbul {
                    return Err(FailureKind::InvalidInstruction);
                }
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let tx_context = host.get_tx_context();
                self.handle_context(ContextKind::ChainId, tx_context, stack)?;
                Ok(None)
            },
            OpCode::SELFBALANCE => {
                // EIP-1884: https://eips.ethereum.org/EIPS/eip-1884
                if exec_context.revision < Revision::Istanbul {
                    return Err(FailureKind::InvalidInstruction);
                }
                let balance = host.get_balance(scope.code_address);
                stack.push(balance)?;
                Self::consume_constant_gas(&mut scope.gas_left, 5)?;
                Ok(None)
            },
            OpCode::BASEFEE => {
                // EIP-3198: https://eips.ethereum.org/EIPS/eip-3198
                if exec_context.revision < Revision::London {
                    return Err(FailureKind::InvalidInstruction);
                }
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let tx_context = host.get_tx_context();
                self.handle_context(ContextKind::BaseFee, tx_context, stack)?;
                Ok(None)
            }
            OpCode::POP => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                stack.pop()?;
                Ok(None)
            },
            OpCode::MLOAD => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let offset = stack.pop()?;
                let gas_consumed = mload(offset, memory, stack, scope.gas_left).map_err(|e| e)?;
                scope.gas_left -= gas_consumed;
                Ok(None)
            },
            OpCode::MSTORE => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let offset = stack.pop()?;
                let gas_consumed = mstore(offset, memory, stack, scope.gas_left)?;
                scope.gas_left -= gas_consumed;
                Ok(None)
            }, 
            OpCode::MSTORE8 => {
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let offset = stack.pop()?;
                let gas_consumed = mstore8(offset, memory, stack, scope.gas_left)?;
                scope.gas_left -= gas_consumed;
                Ok(None)
            },
            OpCode::SLOAD => {
                // static gas cost is 0. dynamic gas cost is deducted on resume.
                let key = stack.pop()?;

                let access_status = if exec_context.revision >= Revision::Berlin {
                    host.access_storage(scope.to, key)
                }else{
                    //  pre-berlin is always warm
                    AccessStatus::Warm
                };
                let value = host.get_storage(scope.to, key);
                stack.push_unchecked(value);

                // calculate dynamic gas
                let gas =
                    if exec_context.revision >= Revision::Berlin {
                        match access_status {
                            AccessStatus::Warm => 100,
                            AccessStatus::Cold => 2100,
                        }
                    }else{
                        match exec_context.revision {
                            Revision::Frontier | Revision::Homestead => 50,
                            Revision::Istanbul => 800,
                            _ => 200
                        }
                    };
                Self::consume_constant_gas(&mut scope.gas_left, gas)?;

                Ok(None)
            },
            OpCode::SSTORE => {
                // https://eips.ethereum.org/EIPS/eip-214
                if exec_context.revision >= Revision::Byzantium {
                    if scope.is_staticcall {
                        return Err(FailureKind::StaticModeViolation)
                    }
                }

                // https://eips.ethereum.org/EIPS/eip-2200
                if exec_context.revision >= Revision::Istanbul {
                    if scope.gas_left <= 2300 {
                        return Err(FailureKind::OutOfGas)
                    }
                }

                // static gas cost is 0. dynamic gas cost is deducted on resume.
                let key = stack.pop()?;
                let new_value = stack.pop()?;

                let access_status = if exec_context.revision >= Revision::Berlin {
                    host.access_storage(scope.to, key)
                }else{
                    //  pre-berlin is always warm
                    AccessStatus::Warm
                };
                let storage_status = host.set_storage(scope.to, key, new_value);

                exec_context.refund_counter += calc_sstore_gas_refund(new_value, exec_context.revision, storage_status);
                let gas = calc_sstore_gas_cost(new_value, exec_context.revision, access_status, storage_status);
                Self::consume_constant_gas(&mut scope.gas_left, gas)?;

                Ok(None)
            },
            OpCode::JUMP => {
                Self::consume_constant_gas(&mut scope.gas_left, 8)?;
                let dest = stack.pop()?;
                let dest = dest.as_usize();
                if dest < scope.code.0.len() && scope.code.0[dest] == OpCode::JUMPDEST.to_u8() {
                    scope.pc = dest;
                }else{
                    return Err(FailureKind::BadJumpDestination);
                }
                Ok(Some(Interrupt::Jump))
            },
            OpCode::JUMPI => {
                Self::consume_constant_gas(&mut scope.gas_left, 10)?;
                let dest = stack.pop()?;
                let dest = dest.as_usize();
                let cond = stack.pop()?;
                if !cond.is_zero() {
                    if dest < scope.code.0.len() && scope.code.0[dest] == OpCode::JUMPDEST.to_u8() {
                        scope.pc = dest;
                        return Ok(Some(Interrupt::Jump));
                    }else{
                        return Err(FailureKind::BadJumpDestination);
                    }
                }
                Ok(None)
            },
            OpCode::PC => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let pc = U256::from(scope.pc);
                stack.push(pc)?;
                Ok(None)
            }
            OpCode::MSIZE => {
                Self::consume_constant_gas(&mut scope.gas_left, 2)?;
                let len = U256::from(memory.0.len());
                stack.push(len)?;
                Ok(None)
            },
            OpCode::JUMPDEST => {
                Self::consume_constant_gas(&mut scope.gas_left, 1)?;
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
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
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
                Self::consume_constant_gas(&mut scope.gas_left, 3)?;
                let offset = opcode.to_usize() - OpCode::SWAP1.to_usize() + 1;
                stack.swap(offset)?;
                Ok(None)
            },

            // OpCode::LOG0 => {
            //     Ok(None)
            // },
            // OpCode::LOG1 => {
            //     Ok(None)
            // },
            // OpCode::LOG2 => {
            //     Ok(None)
            // },
            // OpCode::LOG3 => {
            //     Ok(None)
            // },
            // OpCode::LOG4 => {
            //     Ok(None)
            // },

            // OpCode::CREATE => {
            //     Ok(None)
            // },
            OpCode::CALL => {
                let gas = scope.stack.pop()?;
                let gas = gas.as_u32() as i64;
                let address = scope.stack.pop()?;
                let address = u256_to_address(address);
                let value = scope.stack.pop()?;
                let args_offset = scope.stack.pop()?;
                let args_offset = args_offset.as_usize();
                let args_size = scope.stack.pop()?;
                let args_size = args_size.as_usize();
                let ret_offset = scope.stack.pop()?;
                let ret_offset = ret_offset.as_usize();
                let ret_size = scope.stack.pop()?;
                let ret_size = ret_size.as_usize();

                if !value.is_zero() && exec_context.revision >= Revision::Byzantium && scope.is_staticcall {
                    return Err(FailureKind::StaticModeViolation);
                }

                let static_cost = 
                    if exec_context.revision >= Revision::Berlin {
                        0
                    }else{
                        if exec_context.revision <= Revision::Homestead {
                            40
                        }else{
                            700
                        }
                    };

                let args_cost = resize_memory(args_offset, args_size, &mut scope.memory, scope.gas_left)?;
                let ret_cost = resize_memory(ret_offset, ret_size, &mut scope.memory, scope.gas_left)?;
                let positive_value_cost = 
                    if value.is_zero() {
                        0
                    }else{
                        9000
                    };
                
                let address_access_cost =
                    if exec_context.revision >= Revision::Berlin {
                        match host.access_account(address) {
                            AccessStatus::Cold => 2600,
                            AccessStatus::Warm => 100, 
                        }
                    }else{
                        0
                    };
                
                let value_to_empty_cost = 
                    if host.account_exists(address){
                        0
                    }else{
                        25000
                    };

                let caller_balance = host.get_balance(scope.caller);
                if caller_balance < value {
                    return Err(FailureKind::InsufficientBalance);
                }
                host.subtract_balance(scope.caller, value);
                host.add_balance(address, value);

                let memory_expansion_cost = args_cost + ret_cost;
                let extra_gas = address_access_cost + positive_value_cost + value_to_empty_cost;

                let gas =
                    if exec_context.revision < Revision::Tangerine {
                        gas
                    }else{
                        // https://github.com/ethereum/EIPs/blob/master/EIPS/eip-150.md
                        min(gas, Self::max_call_gas(scope.gas_left - extra_gas))
                    };

                let total_cost = static_cost + gas + extra_gas + memory_expansion_cost;
                Self::consume_constant_gas(&mut scope.gas_left, total_cost)?;

                // gas stipend is added out of thin air
                let gas = gas + if !value.is_zero() { 2300 } else { 0 };

                let params = CallParams {
                    kind: CallKind::Call,
                    gas,
                    address,
                    value,
                    args_offset,
                    args_size,
                    ret_offset,
                    ret_size,
                };

                Ok(Some(Interrupt::Call(params)))
            },
            OpCode::CALLCODE => {
                let gas = scope.stack.pop()?;
                let address = scope.stack.pop()?;
                let address = u256_to_address(address);
                let value = scope.stack.pop()?;
                let args_offset = scope.stack.pop()?;
                let args_size = scope.stack.pop()?;
                let ret_offset = scope.stack.pop()?;
                let ret_size = scope.stack.pop()?;
                let params = CallParams {
                    kind: CallKind::CallCode,
                    gas: gas.as_u32() as i64,
                    address,
                    value,
                    args_offset: args_offset.as_usize(),
                    args_size: args_size.as_usize(),
                    ret_offset: ret_offset.as_usize(),
                    ret_size: ret_size.as_usize(),
                };

                Ok(Some(Interrupt::Call(params)))
            },

            OpCode::RETURN => {
                let offset = stack.pop()?;
                let size = stack.pop()?;
                let (gas_consumed, data) = ret(offset, size, memory, scope.gas_left)?;
                scope.gas_left -= gas_consumed;
                exec_context.return_data_buffer = data.clone();
                Ok(Some(Interrupt::Return(scope.gas_left, data)))
            },

            OpCode::DELEGATECALL => {
                // EIP-7: https://eips.ethereum.org/EIPS/eip-7
                if exec_context.revision < Revision::Homestead {
                    return Err(FailureKind::InvalidInstruction);
                }
                let gas = scope.stack.pop()?;
                let address = scope.stack.pop()?;
                let address = u256_to_address(address);
                let args_offset = scope.stack.pop()?;
                let args_size = scope.stack.pop()?;
                let ret_offset = scope.stack.pop()?;
                let ret_size = scope.stack.pop()?;
                let params = CallParams {
                    kind: CallKind::DelegateCall,
                    gas: gas.as_u32() as i64,
                    address,
                    value: U256::zero(),
                    args_offset: args_offset.as_usize(),
                    args_size: args_size.as_usize(),
                    ret_offset: ret_offset.as_usize(),
                    ret_size: ret_size.as_usize(),
                };

                Ok(Some(Interrupt::Call(params)))
            },
            // OpCode::CREATE2 => {
            //     Ok(None)
            // },
            OpCode::STATICCALL => {
                // EIP-214: https://eips.ethereum.org/EIPS/eip-214
                if exec_context.revision < Revision::Byzantium {
                    return Err(FailureKind::InvalidInstruction);
                }
                let gas = scope.stack.pop()?;
                let address = scope.stack.pop()?;
                let address = u256_to_address(address);
                let args_offset = scope.stack.pop()?;
                let args_size = scope.stack.pop()?;
                let ret_offset = scope.stack.pop()?;
                let ret_size = scope.stack.pop()?;
                let params = CallParams {
                    kind: CallKind::StaticCall,
                    gas: gas.as_u32() as i64,
                    address,
                    value: U256::zero(),
                    args_offset: args_offset.as_usize(),
                    args_size: args_size.as_usize(),
                    ret_offset: ret_offset.as_usize(),
                    ret_size: ret_size.as_usize(),
                };

                Ok(Some(Interrupt::Call(params)))
            },

            // OpCode::REVERT => {
            //     Ok(None)
            // },

            // OpCode::INVALID => {
            //     Ok(None)
            // },

            // OpCode::SELFDESTRUCT => {
            //     Ok(None)
            // },

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

    fn handle_context(&self, kind: ContextKind, context: TxContext, stack: &mut Stack) -> Result<(), FailureKind> {
        match kind {
            ContextKind::Coinbase => {
                let coinbase = address_to_u256(context.coinbase);
                stack.push(coinbase)?;
            },
            ContextKind::Timestamp => {
                let timestamp = U256::from(context.block_timestamp);
                stack.push(timestamp)?;
            },
            ContextKind::Number => {
                let number = U256::from(context.block_number);
                stack.push(number)?;
            },
            ContextKind::Difficulty => {
                stack.push(context.difficulty)?;
            },
            ContextKind::GasLimit => {
                let gas_limit = U256::from(context.gas_limit);
                stack.push(gas_limit)?;
            },
            ContextKind::GasPrice => {
                let gas_price = U256::from(context.gas_price);
                stack.push(gas_price)?;
            },
            ContextKind::ChainId => {
                stack.push(context.chain_id)?;
            },
            ContextKind::BaseFee => {
                stack.push(context.base_fee)?;
            }
        };

        Ok(())
    }

    fn max_call_gas(gas: i64) -> i64 {
        gas - (gas / 64)
    }
}