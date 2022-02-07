use ethereum_types::Address;
use arrayvec::ArrayVec;

use crate::model::{
    code::Code,
};
use crate::interpreter::{
    stack::{Stack,Memory}
};

#[derive(Clone, Debug)]
pub struct CallContext {
    pub pc: usize,
    pub stack: Stack,
    pub memory: Memory,
    pub code: Code,
    pub sender: Address,
    pub to: Address
}

const SIZE: usize = 1024;

/// evm execution call stack
#[derive(Clone, Debug)]
pub struct CallStack(pub ArrayVec<CallContext, SIZE>);

impl CallStack {

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, value: CallContext) {
        unsafe {
            self.0.push_unchecked(value);
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
            return Ok(top);
        }
        Err(CallStackOperationError::StackUnderflow)
    }
}


pub enum CallStackOperationError {
    StackUnderflow,
    StackOverflow,
}