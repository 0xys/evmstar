pub mod host;
pub mod stateful;

use ethereum_types::{Address, U256};
use bytes::Bytes;

use crate::model::evmc::{
    Message, Output, TxContext, AccessStatus, StorageStatus
};

/// EVMC Host interface
/// https://evmc.ethereum.org/structevmc__host__interface.html
pub trait Host {
    fn account_exists(&self, address: Address) -> bool;
    fn get_storage(&self, address: Address, key: U256) -> U256;

    // slightly modified StorageStatus struct for the ease of gas cost/refund calculation
    fn set_storage(&mut self, address: Address, key: U256, value: U256) -> StorageStatus;
    
    fn get_balance(&self, address: Address) -> U256;
    fn get_code_size(&self, address: Address) -> U256;
    fn get_code_hash(&self, address: Address) -> U256;
    fn copy_code(&self, address: Address, code_offset: usize, memory_offset: usize, size: usize);
    fn self_destruct(&mut self, address: Address, beneficiary: Address);
    fn call(&mut self, msg: &Message) -> Output;
    fn get_tx_context(&self) -> TxContext;
    fn emit_log(&mut self, address: Address, data: &[u8], topics: &[U256]);
    fn access_account(&mut self, address: Address) -> AccessStatus;
    fn access_storage(&mut self, address: Address, key: U256) -> AccessStatus;

    // extensions
    fn get_blockhash(&self, height: usize) -> U256;
    fn get_code(&self, address: Address, offset: usize, size: usize) -> Bytes;
    fn add_balance(&mut self, address: Address, amount: U256);
    fn subtract_balance(&mut self, address: Address, amount: U256);
    fn force_set_storage(&mut self, address: Address, key: U256, value: U256);
}