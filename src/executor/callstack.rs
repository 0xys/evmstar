use ethereum_types::{Address, U256};
use arrayvec::ArrayVec;

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

#[derive(Clone, Debug, Default)]
pub struct CallContext {
    pub pc: usize,
    pub stack: Stack,
    pub memory: Memory,
    pub calldata: Calldata,
    pub code: Code,
    pub caller: Address,
    pub to: Address,
    pub origin: Address,
    pub value: U256,
    pub is_staticcall: bool,
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

    pub fn push(&mut self, value: CallContext) {
        unsafe {
            self.0.push_unchecked(Box::new(value));
        }
    }

    pub fn peek(&self) -> Result<&CallContext, CallStackOperationError> {
        if self.is_empty() {
            return Err(CallStackOperationError::StackUnderflow);
        }
        Ok(&self.0[self.0.len() - 1])
    }

    pub fn pop(&mut self) -> Result<CallContext, CallStackOperationError> {
        if let Some(top) = self.0.pop() {
            return Ok(*top);
        }
        Err(CallStackOperationError::StackUnderflow)
    }
}


pub enum CallStackOperationError {
    StackUnderflow,
    StackOverflow,
}