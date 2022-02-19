use serde::Serialize;
use arrayvec::ArrayVec;
use ethereum_types::U256;
use hex::{decode};

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

impl Calldata {
    pub fn get_word(&self, offset: usize) -> U256 {
        if offset + 32 < self.0.len() {
            let word = &self.0[offset..offset+32];
            U256::from_big_endian(word)
        }else{
            let word = &self.0[offset..self.0.len()];
            let padding_size = 32 - word.len();
            let mut word = Vec::from(word);
            for _ in 0..padding_size {
                word.push(0u8);
            }
            U256::from_big_endian(word.as_slice())
        }
    }

    pub fn get_range(&self, offset: usize, size: usize) -> Vec<u8> {
        if offset + size < self.0.len() {
            let data = &self.0[offset..offset+size];
            Vec::from(data)
        }else{
            let word = &self.0[offset..self.0.len()];
            let padding_size = size - word.len();
            let mut word = Vec::from(word);
            for _ in 0..padding_size {
                word.push(0u8);
            }
            word
        }
    }
}

impl From<&str> for Calldata {
    fn from(hex: &str) -> Self {
        let hex = decode(hex).unwrap();
        Self {
            0: Vec::from(hex)
        }
    }
}

impl From<Vec<u8>> for Calldata {
    fn from(data: Vec<u8>) -> Self {
        Self {
            0: data
        }
    }
}

#[test]
fn test_calldata(){
    let calldata = Calldata{
        0: vec![
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,1,

            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,2,

            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,

            0,0,0,3,0,0,0,0,

            0,0,0,0,0,0,0,0,
        ]
    };

    let word = calldata.get_word(0);
    assert_eq!(U256::from(1), word);

    let word = calldata.get_word(32);
    assert_eq!(U256::from(2), word);

    let word = calldata.get_word(68);
    assert_eq!(U256::from(3), word);

    let expected = vec![
        3,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,
    ];
    let word = calldata.get_word(99);

    assert_eq!(U256::from_big_endian(expected.as_slice()), word);

}

#[test]
fn test_get_range() {
    let calldata = Calldata{
        0: vec![
            1,2,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,1,

            0,0,0,0,1,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,0,
            0,0,0,0,2,2,2,2,

            2,0,0,0,0,0,0,0,
            0,0,0,0,0,0,0,3,
        ]
    };

    let data = calldata.get_range(0, 2);
    assert_eq!(vec![1,2], data);

    let expected = vec![
        2,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,1,0,
    ];
    let data = calldata.get_range(1, 32);
    assert_eq!(expected, data);

    let expected = vec![
        0,0,0,0,1,0,0,0,
        0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,
        0,0,0,0,2,2,2,2,

        2,
    ];
    let data = calldata.get_range(32, 33);
    assert_eq!(expected, data);

    let expected = vec![
        2,2,2,2,

        2,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,3,
        0,0,0,0,0,0,0,0,
        0,0,0,0,
    ];
    let data = calldata.get_range(60, 32);
    assert_eq!(expected, data);
}