use crate::host::Host;
use crate::executor::callstack::{
    CallStack, CallContext, ExecutionContext
};
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
}

impl Executor {
    pub fn new(host: Box<dyn Host>) -> Self {
        Self {
            host: host,
            interpreter: Interpreter::default(),
            callstack: CallStack::default(),
            revision: Revision::Shanghai
        }
    }
    pub fn new_with_tracing(host: Box<dyn Host>) -> Self {
        Self {
            host: host,
            interpreter: Interpreter::new_with_tracing(),
            callstack: CallStack::default(),
            revision: Revision::Shanghai
        }
    }

    pub fn call_message(&mut self, msg: &Message) -> Output {
        self.host.call(msg)
    }

    pub fn execute_raw(&mut self, code: &Code) -> Output {
        let mut context = CallContext::default();
        context.code = code.clone();

        let mut resume = Resume::Init;
        let mut gas_left = i64::max_value();    // TODO

        let mut exec_context = ExecutionContext::default();

        loop {
            let interrupt = self.interpreter.resume_interpret(resume, &mut context, &mut exec_context, &mut gas_left);
            
            let interrupt = match interrupt {
                Ok(i) => i,
                Err(failure_kind) => {
                    return Output::new_failure(failure_kind);
                }
            };

            match interrupt {
                Interrupt::Return(gas_left, data) => {
                    return Output::new_success(gas_left, data);
                }
                _ => ()
            };

            resume = self.handle_interrupt(&interrupt);
        }
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