pub mod interpreter;
pub mod stack;
pub mod utils;

use ethereum_types::{
    Address, U256
};
use bytes::Bytes;

use crate::model::evmc::{
    TxContext,
    AccessStatus,
    StorageStatus,
};

pub type StorageKey = U256;
pub type StorageValue = U256;
pub type LogData = Vec<u8>;
pub type LogTopics = Vec<U256>;

#[derive(Clone, Debug, PartialEq)]
pub enum Interrupt {
    AccountExists(Address),
    SelfBalance(Address),
    GetStorage(Address, StorageKey),
    SetStorage(Address, StorageKey, StorageValue),
    GetExtCodeSize(Address),
    GetExtCode(Address, usize, usize, usize),
    GetExtCodeHash(Address),
    CopyCode(Address, usize),
    SelfDestruct(Address, Address),
    Emit(Address, LogData, LogTopics),
    AccessAccount(Address),
    AccessStorage(Address, StorageKey),
    Context(ContextKind),
    Jump,

    Blockhash(usize),

    Call(CallParams),

    Return(i64, Bytes),
    Stop(i64),
    Revert(i64),
}

pub enum Resume {
    Init,
    SelfBalance(U256),
    Context(ContextKind, TxContext),
    GetStorage(StorageValue, AccessStatus),
    SetStorage(StorageValue, AccessStatus, StorageStatus),
    Blockhash(U256),
    GetExtCodeSize(U256, AccessStatus),
    GetExtCode(Bytes, AccessStatus, usize),
    GetExtCodeHash(U256, AccessStatus),
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
    Plain,
    CallCode,
    DelegateCall,
    StaticCall,
}

impl Default for CallKind {
    fn default() -> Self {
        CallKind::Plain
    }
}