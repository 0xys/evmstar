pub mod interpreter;
pub mod stack;
pub mod utils;

use ethereum_types::{
    Address, U256
};
use bytes::Bytes;

pub type StorageKey = U256;
pub type StorageValue = U256;
pub type LogData = Vec<u8>;
pub type LogTopics = Vec<U256>;

#[derive(Clone, Debug, PartialEq)]
pub enum Interrupt {
    SelfDestruct(Address, Address),
    Emit(Address, LogData, LogTopics),
    Jump,

    Call(CallParams),

    Exit(i64, Bytes, ExitKind),
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum ExitKind {
    Stop,
    Return,
    Revert,
}

pub enum Resume {
    Init,
    Returned(bool),
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContextKind {
    Coinbase,
    Timestamp,
    Number,
    Difficulty,
    GasPrice,
    GasLimit,
    ChainId,
    BaseFee,
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct CallParams {
    pub kind: CallKind,
    pub gas: i64,
    pub address: Address,
    pub value: U256,
    pub args_offset: usize,
    pub args_size: usize,
    pub ret_offset: usize,
    pub ret_size: usize,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CallKind {
    Call,
    CallCode,
    DelegateCall,
    StaticCall,
}

impl Default for CallKind {
    fn default() -> Self {
        CallKind::Call
    }
}