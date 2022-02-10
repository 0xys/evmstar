use ethereum_types::{Address, U256, H256};
use bytes::Bytes;

use crate::host::Host;
use crate::model::evmc::{
    Message, Output, TxContext, AccessStatus, StatusCode,
};

/// host without no persistent storage
pub struct TransientHost {

}

impl TransientHost {
    pub fn new() -> Self {
        TransientHost{}
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
    fn set_storage(&mut self, address: Address, key: U256, value: U256) {
    }
    fn get_balance(&self, address: Address) -> U256 {
        U256::max_value()
    }
    fn get_code_size(&self, address: Address) -> usize {
        0
    }
    fn get_code_hash(&self, address: Address) -> H256 {
        H256::zero()
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
            size: 0
        }
    }
    fn get_tx_context(&self) -> TxContext {
        TxContext {
            base_fee: U256::zero(),
            block_number: 0,
            block_timestamp: 0,
            chain_id: U256::one(),
            coinbase: Address::zero(),
            difficulty: U256::zero(),
            gas_limit: 0,
            gas_price: U256::zero(),
            origin: Address::zero(),

        }
    }
    fn emit_log(&mut self, address: Address, data: &[u8], topics: &[U256]) {
    }
    fn access_account(&mut self, address: Address) -> AccessStatus {
        AccessStatus::Warm
    }
    fn access_storage(&mut self, address: Address, key: U256) -> AccessStatus {
        AccessStatus::Warm
    }
}