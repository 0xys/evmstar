use ethereum_types::{Address, U256};
use bytes::Bytes;

use crate::executor::journal::Snapshot;
use crate::host::Host;
use crate::model::code::Code;
use crate::model::evmc::{
    Message, Output, TxContext, AccessStatus, StatusCode, StorageStatus
};

use super::stateful::Account;

/// host without no persistent storage
pub struct TransientHost {
    context: TxContext,
}

impl TransientHost {
    pub fn new() -> Self {
        TransientHost{
            context: TxContext {
                base_fee: U256::zero(),
                block_number: 0,
                block_timestamp: 0,
                chain_id: U256::one(),
                coinbase: Address::zero(),
                difficulty: U256::zero(),
                gas_limit: 0,
                gas_price: U256::zero(),
                origin: Address::zero(),
            },
        }
    }

    pub fn new_with(context: TxContext) -> Self {
        TransientHost{
            context: context,
        }
    }
}

#[allow(unused_variables)]
impl Host for TransientHost {
    fn account_exists(&self, address: Address) -> bool {
        true
    }
    fn get_storage(&self, address: Address, key: U256) -> U256 {
        U256::zero()
    }
    fn set_storage(&mut self, address: Address, key: U256, value: U256) -> StorageStatus {
        StorageStatus::default()
    }
    fn get_balance(&self, address: Address) -> U256 {
        U256::max_value()
    }
    fn get_code_size(&self, address: Address) -> U256 {
        U256::zero()
    }
    fn get_code_hash(&self, address: Address) -> U256 {
        U256::zero()
    }
    fn copy_code(&self, address: Address, code_offset: usize, memory_offset: usize, size: usize) {
    }
    fn self_destruct(&mut self, address: Address, beneficiary: Address) {
    }
    fn call(&mut self, msg: &Message) -> Output {
        Output {
            gas_left: i64::max_value(),
            status_code: StatusCode::Success,
            create_address: None,
            data: Bytes::default(),
            size: 0,
            gas_refund: 0,
            effective_gas_refund: 0,
        }
    }
    fn get_tx_context(&self) -> TxContext {
        self.context.clone()
    }
    fn emit_log(&mut self, address: Address, data: &[u8], topics: &[U256]) {
    }
    fn access_account(&mut self, address: Address) -> AccessStatus {
        AccessStatus::Warm
    }
    fn access_storage(&mut self, address: Address, key: U256) -> AccessStatus {
        AccessStatus::Warm
    }

    fn add_account(&mut self, address: Address, account: Account) {
        
    }
    fn debug_get_storage(&self, address: Address, key: U256) -> U256 {
        U256::zero()
    }
    fn debug_set_storage(&mut self, address: Address, key: U256, new_value: U256) {

    }
    fn debug_set_storage_as_warm(&mut self) {

    }
    fn debug_deploy_contract(&mut self, address_hex: &str, code: Code, balance: U256) {

    }
    fn debug_deploy_contract2(&mut self, address: Address, code: Code, balance: U256) {
        
    }

    fn get_blockhash(&self, height: usize) -> U256 {
        U256::from(0x0101)
    }
    fn get_code(&self, address: Address, offset: usize, size: usize) -> Bytes {
        Bytes::default()
    }
    fn add_balance(&mut self, address: Address, amount: U256){

    }
    fn subtract_balance(&mut self, address: Address, amount: U256){

    }
    fn take_snapshot(&self) -> Snapshot {
        0
    }
    fn rollback(&mut self, snapshot: Snapshot){
        
    }
    fn force_update_storage(&mut self, address: Address, key: U256, value: U256){

    }
}