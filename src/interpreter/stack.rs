use serde::Serialize;
use arrayvec::ArrayVec;
use ethereum_types::U256;

const SIZE: usize = 1024;

/// EVM stack
#[derive(Clone, Debug, Default, Serialize)]
pub struct Stack(pub ArrayVec<U256, SIZE>);

impl Stack {

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, value: U256) {
        unsafe {
            self.0.push_unchecked(value);
        }
    }

    pub fn peek(&self) -> Result<U256, StackOperationError> {
        if self.is_empty() {
            return Err(StackOperationError::StackUnderflow);
        }
        Ok(self.0[self.0.len() - 1])
    }

    pub fn pop(&mut self) -> Result<U256, StackOperationError> {
        if let Some(top) = self.0.pop() {
            return Ok(top);
        }
        Err(StackOperationError::StackUnderflow)
    }

    pub fn swap(&mut self, index: usize) -> Result<(), StackOperationError> {
        if index >= self.0.len() {
            return Err(StackOperationError::StackOverflow);
        }

        let top_index = self.0.len() - 1;
        let index = self.len() - index + 1;
        self.0.swap(index, top_index);

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum StackOperationError {
    StackUnderflow,
    StackOverflow,
}

/// EVM memory
#[derive(Clone, Debug, Default, Serialize)]
pub struct Memory(pub Vec<u8>);

impl Memory {
    pub fn get(&self, offset: usize) -> u8 {
        self.0[offset]
    }

    pub fn get_range<'a>(&'a self, offset: usize, size: usize) -> &'a [u8] {
        &self.0[offset..offset+size]
    }

    pub fn set(&mut self, index: usize, value: u8) {
        self.0[index] = value;
    }

    pub fn set_range(&mut self, offset: usize, value: &[u8]) {
        for i in offset..offset + value.len() {
            self.0[i] = value[i - offset];
        }
    }
}

/// EVM calldata
#[derive(Clone, Debug, Default, Serialize)]
pub struct Calldata(pub Vec<u8>);