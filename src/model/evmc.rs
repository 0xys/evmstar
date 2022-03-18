use ethereum_types::{Address, U256};
use bytes::Bytes;
use std::collections::HashMap;

/// https://evmc.ethereum.org/structevmc__message.html
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub flags: u32,
    pub depth: i32,
    pub gas: i64,
    pub sender: Address,
    pub recipient: Address,
    pub data: Vec<u8>,
    pub value: U256,
    pub create2_salt: U256,
    pub code_address: Address,
}

#[derive(Clone, Debug, Default)]
pub struct AccessList{
    /// map to (count, vec) tuple.
    /// 
    /// count: counts the same address.
    pub map: HashMap<Address, (usize, Vec<U256>)>
}

impl AccessList {
    /// add account to access list.
    /// if address already exists, increment duplicate count.
    pub fn add_account(&mut self, address: Address) {
        if let Some(value) = self.map.get_mut(&address) {
            let next_count = value.0 + 1;
            self.map.insert(address, (next_count, Vec::new()));
        }else{
            self.map.insert(address, (1, Vec::new()));
        }
    }

    /// add storage key to access list.
    pub fn add_storage(&mut self, address: Address, key: U256) {
        if let Some(value) = self.map.get_mut(&address) {
            value.1.push(key);
        }else{
            let mut vec = Vec::new();
            vec.push(key);
            self.map.insert(address, (1, vec));
        }
    }

    /// count the total number of account keys.
    pub fn get_account_count(&self) -> usize {
        let mut count = 0;
        for account in self.map.iter() {
            count += account.1.0;
        }
        count
    }

    /// count the total numer of storage keys.
    pub fn get_storage_count(&self) -> usize {
        let mut count = 0;
        for account in self.map.iter() {
            count += account.1.1.len();
        }
        count
    }
}


/// https://evmc.ethereum.org/structevmc__result.html
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub status_code: StatusCode,
    pub gas_left: i64,
    pub data: Bytes,
    pub size: usize,
    pub create_address: Option<Address>,

    pub gas_refund: i64,            // extension
    pub effective_gas_refund: i64,  // extension
}
impl Default for Output {
    fn default() -> Self {
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
}
impl Output {
    pub fn new_success(gas_left: i64, gas_refund: i64, effective_gas_refund: i64, data: Bytes) -> Self {
        let size = data.len();
        Output {
            gas_left: gas_left,
            status_code: StatusCode::Success,
            create_address: None,
            data: data,
            size: size,
            gas_refund: gas_refund,
            effective_gas_refund: effective_gas_refund,
        }
    }

    pub fn new_failure(failure_kind: FailureKind, gas_left: i64) -> Self {
        Output {
            gas_left: gas_left,
            status_code: StatusCode::Failure(failure_kind),
            create_address: None,
            data: Bytes::default(),
            size: 0,
            gas_refund: 0,
            effective_gas_refund: 0,
        }
    }

    pub fn new_revert(gas_left: i64, data: Bytes) -> Self {
        let size = data.len();
        Output {
            gas_left: gas_left,
            status_code: StatusCode::Failure(FailureKind::Revert),
            create_address: None,
            data: data,
            size: size,
            gas_refund: 0,
            effective_gas_refund: 0,
        }
    }
}


/// https://evmc.ethereum.org/group__EVMC.html#ga4c0be97f333c050ff45321fcaa34d920
#[derive(Clone, Debug, PartialEq)]
pub enum StatusCode {
    Success,
    Failure(FailureKind),
}
#[derive(Clone, Debug, PartialEq)]
pub enum FailureKind {
    Generic(String),
    Revert,
    OutOfGas,
    InvalidInstruction,
    UndefinedInstruction,
    StackOverflow,
    StackUnderflow,
    BadJumpDestination,
    InvalidMemoryAccess,
    CallDepthExceeded,
    StaticModeViolation,
    PrecompileFailure,
    ContractValidationFailure,
    ArgumentOutOfRange,
    WasmUnreachableInstruction,
    WasmTrap,
    InsufficientBalance,
    InternalError(String),
    Rejected,
    OutOfMemory,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StorageStatusKind {
    Added,
    Modified,
    ModifiedAgain,
    Unchanged,
    Deleted,
}
impl Default for StorageStatusKind {
    fn default() -> Self {
        Self::Unchanged
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StorageStatus {
    pub original: U256,
    pub current: U256,
    pub kind: StorageStatusKind,
}

/// https://evmc.ethereum.org/structevmc__tx__context.html
#[derive(Clone, Debug, PartialEq)]
pub struct TxContext {
    pub gas_price: U256,
    pub origin: Address,
    pub coinbase: Address,
    pub block_number: i64,
    pub block_timestamp: i64,
    pub gas_limit: i64,
    pub difficulty: U256,
    pub chain_id: U256,
    pub base_fee: U256,
}

/// https://evmc.ethereum.org/group__EVMC.html#ga9f71195f3873f9979d81d7a5e1b3aaf0
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccessStatus {
    Cold,
    Warm,
}

impl Default for AccessStatus {
    fn default() -> Self {
        Self::Cold
    }
}

/// https://evmc.ethereum.org/group__EVMC.html#gab2fa68a92a6828064a61e46060abc634
#[derive(Clone, Debug, PartialEq)]
pub enum CallKind {
    Call,
    DelegateCall,
    CallCode,
    Create,
    Create2,
}

#[cfg(test)]
mod test {
    use crate::model::evmc::*;

    fn address_0() -> Address {
        Address::from_low_u64_be(0)
    }
    fn address_1() -> Address {
        Address::from_low_u64_be(1)
    }

    #[test]
    pub fn test_access_list() {
        let mut access_list = AccessList::default();

        access_list.add_account(address_0());
        assert_eq!(1, access_list.get_account_count());

        access_list.add_account(address_1());
        assert_eq!(2, access_list.get_account_count());

        access_list.add_account(address_0()); // duplicate
        assert_eq!(3, access_list.get_account_count());
    }

    #[test]
    pub fn test_access_list_storage() {
        let mut access_list = AccessList::default();

        access_list.add_account(address_0());
        assert_eq!(1, access_list.get_account_count());

        access_list.add_account(address_1());
        assert_eq!(2, access_list.get_account_count());

        access_list.add_storage(address_0(), U256::from(0));
        assert_eq!(2, access_list.get_account_count());
        assert_eq!(1, access_list.get_storage_count());

        access_list.add_storage(address_0(), U256::from(1));
        assert_eq!(2, access_list.get_account_count());
        assert_eq!(2, access_list.get_storage_count());

        access_list.add_storage(address_0(), U256::from(0));    // duplicate
        assert_eq!(2, access_list.get_account_count());
        assert_eq!(3, access_list.get_storage_count());

        access_list.add_storage(address_1(), U256::from(1));
        assert_eq!(2, access_list.get_account_count());
        assert_eq!(4, access_list.get_storage_count());
    }
}