use serde::Serialize;
use arrayvec::ArrayVec;
use ethereum_types::U256;

use crate::model::evmc::{
    FailureKind,
};

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

    pub fn push_unchecked(&mut self, value: U256) {
        unsafe {
            self.0.push_unchecked(value);
        }
    }

    pub fn push(&mut self, value: U256) -> Result<(), FailureKind> {
        if self.len() >= SIZE {
            return Err(FailureKind::StackOverflow);
        }
        unsafe {
            self.0.push_unchecked(value);
        }
        Ok(())
    }

    pub fn peek(&self) -> Result<U256, FailureKind> {
        if self.is_empty() {
            return Err(FailureKind::StackUnderflow);
        }
        Ok(self.0[self.0.len() - 1])
    }

    pub fn peek_at(&self, offset: usize) -> Result<U256, FailureKind> {
        if self.is_empty() || self.0.len() <= offset {
            return Err(FailureKind::StackOverflow)
        }
        Ok(self.0[self.0.len() - 1 - offset])
    }

    pub fn pop(&mut self) -> Result<U256, FailureKind> {
        if let Some(top) = self.0.pop() {
            return Ok(top);
        }
        Err(FailureKind::StackUnderflow)
    }

    pub fn swap(&mut self, index: usize) -> Result<(), FailureKind> {
        if self.is_empty() || index >= self.0.len() {
            return Err(FailureKind::StackOverflow);
        }

        let top_index = self.0.len() - 1;
        let index = self.0.len() - 1 - index;
        self.0.swap(index, top_index);

        Ok(())
    }
}

/// EVM memory
#[derive(Clone, Debug, Default, Serialize)]
pub struct Memory(pub Vec<u8>);

/// The size of the EVM 256-bit word.
pub(crate) const WORD_SIZE: i64 = 32;
pub(crate) const MAX_BUFFER_SIZE: u32 = u32::MAX;

/// Returns number of words what would fit to provided number of bytes,
/// i.e. it rounds up the number bytes to number of words.
pub(crate) fn num_words(size_in_bytes: usize) -> i64 {
    ((size_in_bytes as i64) + (WORD_SIZE - 1)) / WORD_SIZE
}

impl Memory {
    pub fn get(&self, offset: usize) -> u8 {
        self.0[offset]
    }

    pub fn get_range<'a>(&'a self, offset: usize, size: usize) -> &'a [u8] {
        &self.0[offset..offset+size]
    }

    pub fn get_word(&self, offset: usize) -> U256 {
        let word = self.get_range(offset, WORD_SIZE as usize);
        U256::from_big_endian(word)
    }

    pub fn set(&mut self, index: usize, value: u8) {
        self.0[index] = value;
    }

    pub fn set_range(&mut self, offset: usize, value: &[u8]) {
        for i in offset..offset + value.len() {
            self.0[i] = value[i - offset];
        }
    }

    /// number of words used to store current load.
    pub fn num_words(&self) -> i64 {
        num_words(self.0.len())
    }
}

/// EVM calldata
#[derive(Clone, Debug, Default, Serialize)]
pub struct Calldata(pub Vec<u8>);