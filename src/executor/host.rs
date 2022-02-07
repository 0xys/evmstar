// use ethereum_types::{
//     Address, U256,
// }; 

// // Abstraction that exposes host context to EVM.
// pub trait Host {
//     /// Check if an account exists.
//     fn account_exists(&self, address: Address) -> bool;
//     /// Get value of a storage key.
//     ///
//     /// Returns `Ok(U256::zero())` if does not exist.
//     fn get_storage(&self, address: Address, key: U256) -> U256;
//     /// Set value of a storage key.
//     fn set_storage(&mut self, address: Address, key: U256, value: U256) -> StorageStatus;
//     /// Get balance of an account.
//     ///
//     /// Returns `Ok(0)` if account does not exist.
//     fn get_balance(&self, address: Address) -> U256;
//     /// Get code size of an account.
//     ///
//     /// Returns `Ok(0)` if account does not exist.
//     fn get_code_size(&self, address: Address) -> U256;
//     /// Get code hash of an account.
//     ///
//     /// Returns `Ok(0)` if account does not exist.
//     fn get_code_hash(&self, address: Address) -> U256;
//     /// Copy code of an account.
//     ///
//     /// Returns `Ok(0)` if offset is invalid.
//     fn copy_code(&self, address: Address, offset: usize, buffer: &mut [u8]) -> usize;
//     /// Self-destruct account.
//     fn selfdestruct(&mut self, address: Address, beneficiary: Address);
//     /// Call to another account.
//     fn call(&mut self, msg: &Message) -> Output;
//     /// Retrieve transaction context.
//     fn get_tx_context(&self) -> TxContext;
//     /// Get block hash.
//     ///
//     /// Returns `Ok(U256::zero())` if block does not exist.
//     fn get_block_hash(&self, block_number: u64) -> U256;
//     /// Emit a log.
//     fn emit_log(&mut self, address: Address, data: &[u8], topics: &[U256]);
//     /// Mark account as warm, return previous access status.
//     ///
//     /// Returns `Ok(AccessStatus::Cold)` if account does not exist.
//     fn access_account(&mut self, address: Address) -> AccessStatus;
//     /// Mark storage key as warm, return previous access status.
//     ///
//     /// Returns `Ok(AccessStatus::Cold)` if account does not exist.
//     fn access_storage(&mut self, address: Address, key: U256) -> AccessStatus;
// }