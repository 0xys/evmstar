use bytes::Bytes;
use std::cmp::min;

use crate::host::Host;
use crate::executor::callstack::{
    CallStack, CallContext, ExecutionContext
};
use crate::interpreter::{CallParams, CallKind};
use crate::interpreter::stack::{Calldata};
use crate::interpreter::{
    Interrupt,
    interpreter::Interpreter,
    Resume,
};

use crate::model::{
    evmc::*,
    code::Code,
    revision::Revision,
};
#[allow(dead_code)]
pub struct Executor {
    host: Box<dyn Host>,
    interpreter: Interpreter,
    callstack: Box<CallStack>,
    revision: Revision,

    /// if true, gas cost outside of EVM opcode, such as intrinsic cost, calldata cost and access list cost,
    /// will be charged.
    is_execution_cost_on: bool
}

const MAX_CODE_SIZE: usize = 0x6000;

impl Executor {
    pub fn new(host: Box<dyn Host>) -> Self {
        Self {
            host: host,
            interpreter: Interpreter::default(),
            callstack: Box::new(CallStack::default()),
            revision: Revision::Shanghai,
            is_execution_cost_on: false,
        }
    }
    pub fn new_with_tracing(host: Box<dyn Host>) -> Self {
        Self {
            host: host,
            interpreter: Interpreter::new_with_tracing(),
            callstack: Box::new(CallStack::default()),
            revision: Revision::Shanghai,
            is_execution_cost_on: false,
        }
    }
    pub fn new_with(host: Box<dyn Host>, is_trace: bool, revision: Revision) -> Self {
        Self {
            host: host,
            interpreter: match is_trace {
                true => Interpreter::new_with_tracing(),
                false => Interpreter::default()
            },
            callstack: Box::new(CallStack::default()),
            revision: revision,
            is_execution_cost_on: false,
        }
    }

    /// gas cost that is not related to EVM opcode, such as intrinsic cost, calldata cost and access list cost, will be charged.
    pub fn new_with_execution_cost(host: Box<dyn Host>, is_trace: bool, revision: Revision) -> Self {
        Self {
            host: host,
            interpreter: match is_trace {
                true => Interpreter::new_with_tracing(),
                false => Interpreter::default()
            },
            callstack: Box::new(CallStack::default()),
            revision: revision,
            is_execution_cost_on: true,
        }
    }

    pub fn call_message(&mut self, msg: &Message) -> Output {
        self.host.call(msg)
    }

    /// execute with eip-2930 access list provided.
    /// 
    /// https://eips.ethereum.org/EIPS/eip-2930
    pub fn execute_with_access_list(&mut self, mut context: CallContext, access_list: AccessList) -> Output {
        if self.revision < Revision::Berlin {
            panic!("eip2930 is enabled after Berlin onward.");
        }

        for access in access_list.map.into_iter() {
            self.host.access_account(access.0);
            if self.is_execution_cost_on {
                let account_cost = 2400 * access.1.0;
                if !consume_gas(&mut context.gas_left, account_cost as i64){
                    return Output::new_failure(FailureKind::OutOfGas, 0);
                }
            }

            for key in access.1.1 {
                self.host.access_storage(access.0, key);
                if self.is_execution_cost_on {
                    if !consume_gas(&mut context.gas_left, 1900){
                        return Output::new_failure(FailureKind::OutOfGas, 0);
                    }
                }
            }
        }
        self.execute_raw_with(context)
    }

    pub fn execute_raw_with(&mut self, mut context: CallContext) -> Output {
        let mut exec_context = ExecutionContext {
            refund_counter: 0,
            revision: self.revision,
            num_of_selfdestruct: 0,
        };

        if self.revision >= Revision::Spurious {
            // EIP-170: https://eips.ethereum.org/EIPS/eip-170
            if context.code.0.len() > MAX_CODE_SIZE {
                return Output::new_failure(FailureKind::OutOfGas, 0);
            }
        }

        if self.revision >= Revision::Berlin {
            // https://eips.ethereum.org/EIPS/eip-2929#specification
            // accessed_addresses is initialized to include
            // the tx.sender, tx.to (or the address being created if it is a contract creation transaction)
            // and the set of all precompiles.
            self.host.access_account(context.to);
            self.host.access_account(context.caller);
        }

        if self.is_execution_cost_on {
            // intrinsic gas cost deduction
            if !consume_gas(&mut context.gas_left, 21000){
                return Output::new_failure(FailureKind::OutOfGas, 0);
            }

            let calldata_cost = cost_of_calldata(&context.calldata, self.revision);
            // calldata cost deduction
            if !consume_gas(&mut context.gas_left, calldata_cost){
                return Output::new_failure(FailureKind::OutOfGas, 0);
            }
        }

        self.callstack.push(context.clone()).unwrap();

        let mut resume = Resume::Init;
        loop {
            let interrupt = self.interpreter.resume_interpret(resume, &mut context, &mut exec_context);
            
            let interrupt = match interrupt {
                Ok(i) => i,
                Err(failure_kind) => match failure_kind {
                    FailureKind::Revert => return Output::new_failure(failure_kind, context.gas_left),
                    _ => return Output::new_failure(failure_kind, 0),
                }
            };

            match interrupt {
                Interrupt::Return(gas_left, data) => {
                    let src = match self.callstack.pop() {
                        None => panic!("pop from empty callstack is not allowed."),
                        Some(c) => c,
                    };
                    let src = src.borrow_mut();
                    if self.callstack.is_empty() {
                        let effective_refund = calc_effective_refund(context.gas_limit, gas_left, exec_context.refund_counter, exec_context.num_of_selfdestruct, self.revision);
                        return Output::new_success(gas_left, exec_context.refund_counter, effective_refund, data);
                    }
                    let dest = self.callstack.peek();
                    let mut dest = dest.borrow_mut();

                    dest.memory.set_range(src.ret_offset, &data[..src.ret_size]);
                    dest.gas_left += src.gas_left;  // refund unused gas

                    resume = Resume::Returned;
                    continue;
                },
                Interrupt::Stop(gas_left) => {
                    let effective_refund = calc_effective_refund(context.gas_limit, gas_left, exec_context.refund_counter, exec_context.num_of_selfdestruct, self.revision);
                    return Output::new_success(gas_left, exec_context.refund_counter, effective_refund, Bytes::default());
                },
                Interrupt::Call(params) => {
                    match self.push_new_call_context(&params) {
                        Err(kind) => {
                            return Output::new_failure(kind, 0);
                        },
                        _ => (),
                    }
                }
                _ => ()
            };

            resume = self.handle_interrupt(&interrupt);
        }
    }

    pub fn execute_raw(&mut self, code: &Code) -> Output {
        let mut context = CallContext::default();
        context.code = code.clone();
        self.execute_raw_with(context)
    }

    fn handle_interrupt(&mut self, interrupt: &Interrupt) -> Resume {
        match interrupt {
            Interrupt::Balance(address) => {
                let access_status = if self.revision >= Revision::Berlin {
                    self.host.access_account(*address)
                }else{
                    AccessStatus::Warm
                };
                let balance = self.host.get_balance(*address);
                Resume::Balance(balance, access_status)
            },
            Interrupt::SelfBalance(address) => {
                let balance = self.host.get_balance(*address);
                Resume::SelfBalance(balance)
            }
            Interrupt::Context(kind) => {
                let context = self.host.get_tx_context();
                Resume::Context(*kind, context)
            },
            Interrupt::GetExtCodeSize(address) => {
                let access_status = if self.revision >= Revision::Berlin {
                    self.host.access_account(*address)
                }else{
                    AccessStatus::Warm
                };
                let size = self.host.get_code_size(*address);
                Resume::GetExtCodeSize(size, access_status)
            },
            Interrupt::GetExtCode(address, dest_offset, offset, size) => {
                let access_status = if self.revision >= Revision::Berlin {
                    self.host.access_account(*address)
                }else{
                    AccessStatus::Warm
                };
                let code = self.host.get_code(*address, *offset, *size);
                Resume::GetExtCode(code, access_status, *dest_offset)
            },
            Interrupt::GetExtCodeHash(address) => {
                let access_status = self.host.access_account(*address);
                let hash = self.host.get_code_hash(*address);
                Resume::GetExtCodeHash(hash, access_status)
            },
            Interrupt::Blockhash(height) => {
                let hash = self.host.get_blockhash(*height);
                Resume::Blockhash(hash)
            },
            Interrupt::GetStorage(address, key) => {
                let access_status = if self.revision >= Revision::Berlin {
                    self.host.access_storage(*address, *key)
                }else{
                    //  pre-berlin is always warm
                    AccessStatus::Warm
                };
                let value = self.host.get_storage(*address, *key);
                Resume::GetStorage(value, access_status)
            },
            Interrupt::SetStorage(address, key, new_value) => {
                let access_status = if self.revision >= Revision::Berlin {
                    self.host.access_storage(*address, *key)
                }else{
                    //  pre-berlin is always warm
                    AccessStatus::Warm
                };
                let storage_status = self.host.set_storage(*address, *key, *new_value);
                Resume::SetStorage(*new_value, access_status, storage_status)
            },
            Interrupt::Call(_) => {
                Resume::Init
            },
            _ => {
                Resume::Unknown
            }
        }
    }

    fn push_new_call_context(&mut self, params: &CallParams) -> Result<(), FailureKind> {
        let new_context = {
            let current_context = self.callstack.peek();
            let mut current_context = current_context.borrow_mut();
            let (dynamic_cost, new_context) = self.create_call_context(&current_context, params);
        
            if !consume_gas(&mut current_context.gas_left, dynamic_cost) {
                return Err(FailureKind::OutOfGas)
            }
            new_context
        };
                
        self.callstack.push(new_context)?;

        Ok(())
    }

    fn create_call_context(&self, current: &CallContext, params: &CallParams) -> (i64, CallContext) {
        match params.kind {
            CallKind::Plain => {
                let mut context = CallContext::default();
                context.origin = current.origin;
                context.caller = current.code_address;
                context.to = params.address;
                context.code_address = params.address;

                context.calldata = Calldata::default(); // TODO

                let code_size = self.host.get_code_size(params.address);
                context.code = self.host.get_code(params.address, 0, code_size.as_usize()).into();
                
                context.value = params.value;
                
                let mut gas = params.gas;
                if params.value.is_zero() {
                    gas += 2300;    // gas stipend is added out of thin air.
                }

                context.gas_limit = gas;
                context.gas_left = gas;

                context.ret_offset = params.ret_offset;
                context.ret_size = params.ret_size;

                (0, context)
            },
            CallKind::CallCode => {
                let context = CallContext::default();
                (0, context)
            },
            CallKind::StaticCall => {
                let context = CallContext::default();
                (0, context)
            },
            CallKind::DelegateCall => {
                let context = CallContext::default();
                (0, context)
            },
        }
    }
}

fn consume_gas(gas_left: &mut i64, gas: i64) -> bool {
    *gas_left -= gas;
    if *gas_left < 0 {
        return false;
    }
    true
}

fn cost_of_calldata(calldata: &Calldata, revision: Revision) -> i64 {
    let mut cost = 0i64;
    for hex in &calldata.0 {
        cost += 
            if *hex == 0 {
                4
            }else{
                // https://eips.ethereum.org/EIPS/eip-2028
                if revision >= Revision::Istanbul {
                    16
                }else{
                    68
                }
            }
    }
    cost
}

fn calc_effective_refund(
    gas_limit: i64,
    gas_left: i64,
    refund_counter: i64,
    num_of_selfdestruct: i64,
    revision: Revision
) -> i64 {
    let refund = refund_counter + 
        if revision < Revision::London {
            24000 * num_of_selfdestruct
        }else{
            0
        };
    
    let max_refund_quotient =
        if revision > Revision::London {
            5
        }else{
            2
        };
    
    let max_refund = (gas_limit - gas_left) / max_refund_quotient;
    let refund = min(refund, max_refund);
    refund
}