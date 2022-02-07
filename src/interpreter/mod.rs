pub mod interpreter;
pub mod stack;

use ethereum_types::{
    Address, U256
};

pub type StorageKey = U256;
pub type StorageValue = U256;
pub type LogData = Vec<u8>;
pub type LogTopics = Vec<U256>;

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

    Return,
}