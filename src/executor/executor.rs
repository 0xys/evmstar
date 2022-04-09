use bytes::Bytes;
use std::cmp::min;

use std::rc::Rc;
use std::cell::RefCell;

use crate::host::Host;
use crate::executor::callstack::{
    CallStack, CallScope, ExecutionContext
};
use crate::interpreter::{CallParams, CallKind, ExitKind};
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
    host: Rc<RefCell<dyn Host>>,
    interpreter: Interpreter,
    callstack: Box<CallStack>,
    revision: Revision,

    /// if true, gas cost outside of EVM opcode, such as intrinsic cost, calldata cost and access list cost,
    /// will be charged.
    is_execution_cost_on: bool
}

const MAX_CODE_SIZE: usize = 0x6000;
const SUCCESS: bool = true;
const FAILED: bool = false;


impl Executor {
    pub fn new(host: Rc<RefCell<dyn Host>>) -> Self {
        Self {
            host: host,
            interpreter: Interpreter::default(),
            callstack: Box::new(CallStack::default()),
            revision: Revision::Shanghai,
            is_execution_cost_on: false,
        }
    }
    pub fn new_with_tracing(host: Rc<RefCell<dyn Host>>) -> Self {
        Self {
            host: host,
            interpreter: Interpreter::new_with_tracing(),
            callstack: Box::new(CallStack::default()),
            revision: Revision::Shanghai,
            is_execution_cost_on: false,
        }
    }
    pub fn new_with(host: Rc<RefCell<dyn Host>>, is_trace: bool, revision: Revision) -> Self {
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
    pub fn new_with_execution_cost(host: Rc<RefCell<dyn Host>>, is_trace: bool, revision: Revision) -> Self {
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

    pub fn set_revision(&mut self, revision: Revision) {
        self.revision = revision;
    }

    pub fn call_message(&mut self, msg: &Message) -> Output {
        (*self.host).borrow_mut().call(msg)
    }

    /// execute with eip-2930 access list provided.
    /// 
    /// https://eips.ethereum.org/EIPS/eip-2930
    pub fn execute_with_access_list(&mut self, mut scope: CallScope, access_list: AccessList) -> Output {
        if self.revision < Revision::Berlin {
            panic!("eip2930 is enabled after Berlin onward.");
        }

        {
            let mut host = (*self.host).borrow_mut();

            for access in access_list.map.into_iter() {
                host.access_account(access.0);
                if self.is_execution_cost_on {
                    let account_cost = 2400 * access.1.0;
                    if !consume_gas(&mut scope.gas_left, account_cost as i64){
                        return Output::new_failure(FailureKind::OutOfGas, 0);
                    }
                }
    
                for key in access.1.1 {
                    host.access_storage(access.0, key);
                    if self.is_execution_cost_on {
                        if !consume_gas(&mut scope.gas_left, 1900){
                            return Output::new_failure(FailureKind::OutOfGas, 0);
                        }
                    }
                }
            }
        }
        
        self.execute_raw_with(scope)
    }

    pub fn execute_raw_with(&mut self, mut scope: CallScope) -> Output {
        let mut exec_context = ExecutionContext {
            refund_counter: 0,
            revision: self.revision,
            num_of_selfdestruct: 0,
            return_data_buffer: Bytes::default(),
        };

        if self.revision >= Revision::Spurious {
            // EIP-170: https://eips.ethereum.org/EIPS/eip-170
            if scope.code.0.len() > MAX_CODE_SIZE {
                return Output::new_failure(FailureKind::OutOfGas, 0);
            }
        }

        // let mut host = (*self.host).borrow_mut();

        if self.revision >= Revision::Berlin {
            // https://eips.ethereum.org/EIPS/eip-2929#specification
            // accessed_addresses is initialized to include
            // the tx.sender, tx.to (or the address being created if it is a contract creation transaction)
            // and the set of all precompiles.
            (*self.host).borrow_mut().access_account(scope.to);
            (*self.host).borrow_mut().access_account(scope.caller);
        }

        if self.is_execution_cost_on {
            // intrinsic gas cost deduction
            if !consume_gas(&mut scope.gas_left, 21000){
                return Output::new_failure(FailureKind::OutOfGas, 0);
            }

            let calldata_cost = cost_of_calldata(&scope.calldata, self.revision);
            // calldata cost deduction
            if !consume_gas(&mut scope.gas_left, calldata_cost){
                return Output::new_failure(FailureKind::OutOfGas, 0);
            }
        }

        (*self.host).borrow_mut().subtract_balance(scope.caller, scope.value);
        (*self.host).borrow_mut().add_balance(scope.to, scope.value);

        self.callstack.push(scope.clone()).unwrap();

        let mut resume = Resume::Init;
        loop {
            let interrupt = {
                let mut current_scope = self.callstack.peek().borrow_mut(); // current scope is top of the callstack.
                let interrupt = self.interpreter.resume_interpret(resume, &mut current_scope, &mut exec_context, self.host.clone());
                interrupt
            };
            
            match interrupt {
                Err(failure_kind) => {
                    let child = match self.callstack.pop() {
                        None => panic!("pop from empty callstack is not allowed."),
                        Some(scope) => scope,
                    };
                    if self.callstack.is_empty() {
                        match failure_kind {
                            FailureKind::Revert => return Output::new_failure(failure_kind, scope.gas_left),
                            _ => return Output::new_failure(failure_kind, 0),
                        }
                    }
    
                    (*self.host).borrow_mut().rollback(&child.borrow().snapshot);
    
                    resume = Resume::Returned(FAILED);
                    continue;
                },
                Ok(interrupt) => {
                    match interrupt {
                        Interrupt::Exit(gas_left, data, exit_kind) => {
                            if let Some(r) = self.exit_scope(&data, exit_kind) {
                                resume = r;
                                continue;
                            }else{
                                if exit_kind == ExitKind::Revert {
                                    return Output::new_revert(gas_left, data);
                                }
                                let effective_refund = calc_effective_refund(scope.gas_limit, gas_left, exec_context.refund_counter, exec_context.num_of_selfdestruct, self.revision);
                                return Output::new_success(gas_left, exec_context.refund_counter, effective_refund, data);
                            }
                        },
                        Interrupt::Call(params) => {
                            match self.push_child_scope(&params) {
                                Err(kind) => {
                                    return Output::new_failure(kind, 0);
                                },
                                _ => {
                                    // Upon executing any call-like opcode, the buffer is cleared.
                                    // as specified in EIP-211 https://eips.ethereum.org/EIPS/eip-211
                                    exec_context.return_data_buffer = Bytes::default();
                                },
                            }
                        },
                        _ => panic!("unknown interrupt")
                    }
                    resume = self.handle_interrupt(&interrupt);
                },
            }
        }
    
    }

    fn exit_scope(&mut self, data: &Bytes, exit_kind: ExitKind) -> Option<Resume> {
        let child = match self.callstack.pop() {
            None => panic!("pop from empty callstack is not allowed."),
            Some(c) => c,
        };
        let child = child.borrow_mut();

        if exit_kind == ExitKind::Revert {
            (*self.host).borrow_mut().rollback(&child.snapshot); // revert the state to previous snapshot
        }

        if self.callstack.is_empty() {
            return None;
        }
        let parent = self.callstack.peek();
        let mut parent = parent.borrow_mut();

        if exit_kind != ExitKind::Stop {
            parent.memory.set_range(child.ret_offset, &data[..child.ret_size]);
        }
        parent.gas_left = parent.gas_left.saturating_add(child.gas_left);  // refund unused gas

        if exit_kind == ExitKind::Revert {
            return Some(Resume::Returned(FAILED));
        }
        Some(Resume::Returned(SUCCESS))
    }

    pub fn execute_raw(&mut self, code: &Code) -> Output {
        let mut scope = CallScope::default();
        scope.code = code.clone();
        self.execute_raw_with(scope)
    }

    fn handle_interrupt(&mut self, interrupt: &Interrupt) -> Resume {
        match interrupt {
            Interrupt::Call(_) => {
                Resume::Init
            },
            _ => {
                Resume::Unknown
            }
        }
    }

    fn push_child_scope(&mut self, params: &CallParams) -> Result<(), FailureKind> {
        let child = {
            let parent = self.callstack.peek();
            let parent = parent.borrow_mut();
            let child = self.create_child_scope(&parent, params);
            child
        };
        
        self.callstack.push(child)?;

        Ok(())
    }

    fn create_child_scope(&self, parent: &CallScope, params: &CallParams) -> CallScope {
        let host = (*self.host).borrow_mut();
        match params.kind {
            CallKind::Call => {
                let mut child = CallScope::default();
                child.origin = parent.origin;
                child.caller = parent.code_address;
                child.to = params.address;
                child.code_address = params.address;

                child.calldata = parent.memory.get_range(params.args_offset, params.args_size).into();

                let code_size = host.get_code_size(params.address);
                child.code = host.get_code(params.address, 0, code_size.as_usize()).into();
                
                child.value = params.value;
                child.gas_limit = params.gas;
                child.gas_left = params.gas;

                child.ret_offset = params.ret_offset;
                child.ret_size = params.ret_size;

                child.is_staticcall = parent.is_staticcall;    // child succeeds `is_static` flag
                child.snapshot = params.snapshot;

                child
            },
            CallKind::CallCode => {
                let child = CallScope::default();
                child
            },
            CallKind::StaticCall => {
                let child = CallScope::default();
                child
            },
            CallKind::DelegateCall => {
                let child = CallScope::default();
                child
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