use crate::host::Host;
use crate::executor::callstack::{CallStack, CallContext};
use crate::interpreter::{
    Interrupt,
    interpreter::Interpreter
};
use crate::resume::Resume;

use crate::model::{
    evmc::*,
    code::Code,
};
#[allow(dead_code)]
pub struct Executor {
    host: Box<dyn Host>,
    interpreter: Interpreter,
    callstack: CallStack,
}

impl Executor {
    pub fn new(host: Box<dyn Host>) -> Self {
        Self {
            host: host,
            interpreter: Interpreter::default(),
            callstack: CallStack::default(),
        }
    }

    pub fn call_message(&mut self, msg: &Message) -> Output {
        self.host.call(msg)
    }

    pub fn execute_raw(&mut self, code: &Code) -> Output {
        let mut context = CallContext::default();
        context.code = code.clone();

        let mut resume = Resume::Init;
        loop {
            let interrupt = self.interpreter.resume_interpret(resume, &mut context);
            if interrupt.is_err() {
                return Output::default_failure();
            }
            
            let interrupt = interrupt.unwrap();
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
            _ => {
                Resume::Unknown
            }
        }
    }
}