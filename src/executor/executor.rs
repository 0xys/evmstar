use bytes::Bytes;

use crate::host::Host;
use crate::executor::callstack::{
    CallStack, CallContext, ExecutionContext
};
use crate::interpreter::stack::Calldata;
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
    callstack: CallStack,
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
            callstack: CallStack::default(),
            revision: Revision::Shanghai,
            is_execution_cost_on: false,
        }
    }
    pub fn new_with_tracing(host: Box<dyn Host>) -> Self {
        Self {
            host: host,
            interpreter: Interpreter::new_with_tracing(),
            callstack: CallStack::default(),
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
            callstack: CallStack::default(),
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
            callstack: CallStack::default(),
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
        for access in access_list.0.into_iter() {
            self.host.access_account(access.0);
            if self.is_execution_cost_on {
                if !consume_gas(&mut context.gas_left, 2400){
                    return Output::new_failure(FailureKind::OutOfGas, 0);
                }
            }

            for key in access.1 {
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
        let mut resume = Resume::Init;

        let mut exec_context = ExecutionContext {
            refund_counter: 0,
            revision: self.revision
        };

        if self.revision >= Revision::Spurious {
            // EIP-170: https://eips.ethereum.org/EIPS/eip-170
            if context.code.0.len() > MAX_CODE_SIZE {
                return Output::new_failure(FailureKind::OutOfGas, 0);
            }
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
                    return Output::new_success(gas_left, exec_context.refund_counter, data);
                },
                Interrupt::Stop(gas_left) => {
                    return Output::new_success(gas_left, exec_context.refund_counter, Bytes::default())
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
                let balance = self.host.get_balance(*address);
                Resume::Balance(balance)
            },
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
            _ => {
                Resume::Unknown
            }
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