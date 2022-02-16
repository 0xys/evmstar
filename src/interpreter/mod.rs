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
    StorageDiff,
};

pub type StorageKey = U256;
pub type StorageValue = U256;
pub type LogData = Vec<u8>;
pub type LogTopics = Vec<U256>;

#[derive(Clone, Debug, PartialEq)]
pub enum Interrupt {
    AccountExists(Address),
    Balance(Address),
    GetStorage(Address, StorageKey),
    SetStorage(Address, StorageKey, StorageValue),
    GetCodeSize(Address),
    GetCodeHash(Address),
    CopyCode(Address, usize),
    SelfDestruct(Address, Address),
    Call,
    Emit(Address, LogData, LogTopics),
    AccessAccount(Address),
    AccessStorage(Address, StorageKey),
    Context(ContextKind),
    Jump,

    Blockhash(usize),

    Return(i64, Bytes),
}

pub enum Resume {
    Init,
    Balance(U256),
    Context(ContextKind, TxContext),
    GetStorage(StorageValue, AccessStatus),
    SetStorage(StorageValue, AccessStatus, StorageDiff),
    Blockhash(U256),
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