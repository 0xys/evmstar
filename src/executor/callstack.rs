use ethereum_types::{Address, U256};
use arrayvec::ArrayVec;

use crate::model::evmc::FailureKind;
use crate::model::{
    code::Code,
    revision::Revision,
};
use crate::interpreter::{
    stack::{Stack, Memory, Calldata}
};

#[derive(Copy, Clone, Debug, Default)]
pub struct ExecutionContext {
    pub refund_counter: i64,
    pub revision: Revision,
}

#[derive(Clone, Debug)]
pub struct CallContext {
    pub pc: usize,
    pub stack: Stack,
    pub memory: Memory,
    pub calldata: Calldata,
    pub code: Code,
    pub code_address: Address,
    pub caller: Address,
    pub to: Address,
    pub origin: Address,
    pub value: U256,
    pub is_staticcall: bool,
    pub gas_limit: i64,
    pub gas_left: i64,
    pub num_of_selfdestruct: i64,
}
impl Default for CallContext {
    fn default() -> Self {
        CallContext {
            pc: 0,
            stack: Stack::default(),
            memory: Memory::default(),
            calldata: Calldata::default(),
            code: Code::default(),
            code_address: Address::default(),
            caller: Address::default(),
            to: Address::default(),
            origin: Address::default(),
            value: U256::default(),
            is_staticcall: false,
            gas_limit: i64::max_value(),
            gas_left: i64::max_value(),
            num_of_selfdestruct: 0,
        }
    }
}


const SIZE: usize = 1024;

/// evm execution call stack
#[derive(Clone, Debug, Default)]
pub struct CallStack(pub ArrayVec<Box<CallContext>, SIZE>);

impl CallStack {

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, value: CallContext) -> Result<(), FailureKind> {
        if self.0.len() >= SIZE {
            return Err(FailureKind::CallDepthExceeded);
        }
        unsafe {
            self.0.push_unchecked(Box::new(value));
        }
        Ok(())
    }

    pub fn peek(&self) -> CallContext {
        if self.is_empty() {
            panic!("call stack must not be empty");
        }
        (*self.0[self.0.len() - 1]).clone()
    }

    pub fn pop(&mut self) -> Option<CallContext> {
        if let Some(top) = self.0.pop() {
            return Some(*top);
        }
        None
    }
}