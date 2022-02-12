use ethereum_types::{Address, U256};
use bytes::Bytes;
use std::{
    collections::HashMap,
    sync::Mutex,
};

use crate::host::Host;
use crate::model::evmc::{
    Message, Output, TxContext, AccessStatus, StatusCode, StorageStatus
};
use hex_literal::hex;

/// LOG record.
#[derive(Clone, Debug, PartialEq)]
pub struct LogRecord {
    /// The address of the account which created the log.
    pub creator: Address,

    /// The data attached to the log.
    pub data: Bytes,

    /// The log topics.
    pub topics: Vec<U256>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelfdestructRecord {
    /// The address of the account which has self-destructed.
    pub selfdestructed: Address,

    /// The address of the beneficiary account.
    pub beneficiary: Address,
}

#[derive(Clone, Debug, Default)]
pub struct StorageValue {
    pub value: U256,
    pub dirty: bool,
    pub access_status: AccessStatus,
}

#[derive(Clone, Debug, Default)]
pub struct Account {
    /// The account nonce.
    pub nonce: u64,
    /// The account code.
    pub code: Bytes,
    /// The code hash. Can be a value not related to the actual code.
    pub code_hash: U256,
    /// The account balance.
    pub balance: U256,
    /// The account storage map.
    pub storage: HashMap<U256, StorageValue>,
}

const MAX_RECORDED_ACCOUNT_ACCESSES: usize = 200;
// const MAX_RECORDED_CALLS: usize = 100;

#[derive(Clone, Debug, Default)]
pub struct Records {
    /// The copy of call inputs for the recorded_calls record.
    pub call_inputs: Vec<Bytes>,

    pub blockhashes: Vec<u64>,
    pub account_accesses: Vec<Address>,
    pub calls: Vec<Message>,
    pub logs: Vec<LogRecord>,
    pub selfdestructs: Vec<SelfdestructRecord>,
}
impl Records {
    fn record_account_access(&mut self, address: Address) {
        if self.account_accesses.len() < MAX_RECORDED_ACCOUNT_ACCESSES {
            self.account_accesses.push(address)
        }
    }
}

/// mocked host with storage implemented with HashMap
/// https://evmc.ethereum.org/mocked__host_8hpp_source.html
/// https://github.com/vorot93/evmodin/blob/master/src/util/mocked_host.rs
pub struct StatefulHost {
    context: TxContext,
    accounts: HashMap<Address, Account>,
    recorded: Mutex<Records>,
}

impl StatefulHost {
    pub fn new() -> Self {
        Self {
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
            accounts: Default::default(),
            recorded: Mutex::default()
        }
    }

    pub fn new_with(context: TxContext) -> Self {
        Self {
            context: context,
            accounts: Default::default(),
            recorded: Mutex::default()
        }
    }
}

#[allow(unused_variables)]
impl Host for StatefulHost {

    fn account_exists(&self, address: Address) -> bool {
        let mut record = self.recorded.lock().unwrap();
        record.record_account_access(address);
        self.accounts.contains_key(&address)
    }

    fn get_storage(&self, address: Address, key: U256) -> U256 {
        let mut record = self.recorded.lock().unwrap();
        record.record_account_access(address);

        self.accounts
            .get(&address)
            .and_then(|account| account.storage.get(&key).map(|value| value.value))
            .unwrap_or_else(U256::zero)
    }

    fn set_storage(&mut self, address: Address, key: U256, value: U256) -> StorageStatus{
        let mut record = self.recorded.lock().unwrap();
        record.record_account_access(address);

        // Get the reference to the old value.
        // This will create the account in case it was not present.
        // This is convenient for unit testing and standalone EVM execution to preserve the
        // storage values after the execution terminates.
        let old = self
            .accounts
            .entry(address)
            .or_default()
            .storage
            .entry(key)
            .or_default();

        // Follow https://eips.ethereum.org/EIPS/eip-1283 specification.
        // WARNING! This is not complete implementation as refund is not handled here.

        if old.value == value {
            return StorageStatus::Unchanged;
        }

        let status = if !old.dirty {
            old.dirty = true;
            if old.value.is_zero() {
                StorageStatus::Added
            } else if !value.is_zero() {
                StorageStatus::Modified
            } else {
                StorageStatus::Deleted
            }
        } else {
            StorageStatus::ModifiedAgain
        };

        old.value = value;

        status
    }
    
    fn get_balance(&self, address: Address) -> U256 {
        let mut record = self.recorded.lock().unwrap();
        record.record_account_access(address);

        self.accounts
            .get(&address)
            .map(|account| account.balance)
            .unwrap_or_else(U256::zero)
    }

    fn get_code_size(&self, address: Address) -> U256 {
        let mut record = self.recorded.lock().unwrap();
        record.record_account_access(address);

        self.accounts
            .get(&address)
            .map(|account| account.code.len().into())
            .unwrap_or_else(U256::zero)
    }

    fn get_code_hash(&self, address: Address) -> U256 {
        let mut record = self.recorded.lock().unwrap();
        record.record_account_access(address);

        self.accounts
            .get(&address)
            .map(|account| account.code_hash)
            .unwrap_or_else(U256::zero)
    }

    fn copy_code(&self, address: Address, code_offset: usize, memory_offset: usize, size: usize) {
        let mut record = self.recorded.lock().unwrap();
        record.record_account_access(address);
        ()
    }

    fn self_destruct(&mut self, address: Address, beneficiary: Address) {
        let mut record = self.recorded.lock().unwrap();
        record.record_account_access(address);
        record.selfdestructs.push(SelfdestructRecord{
            selfdestructed: address,
            beneficiary
        });
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
        self.context.clone()
    }

    fn emit_log(&mut self, address: Address, data: &[u8], topics: &[U256]) {
        let mut record = self.recorded.lock().unwrap();
        record.logs.push(LogRecord {
            creator: address,
            data: data.to_vec().into(),
            topics: topics.to_vec()
        });
    }

    fn access_account(&mut self, address: Address) -> AccessStatus {
        let mut record = self.recorded.lock().unwrap();
        let already_accessed = record.account_accesses.iter().any(|&a| a == address);

        record.record_account_access(address);

        if address.0 >= hex!("0000000000000000000000000000000000000001") && address.0 <= hex!("0000000000000000000000000000000000000009")
        {
            return AccessStatus::Warm;
        }

        if already_accessed {
            AccessStatus::Warm
        } else {
            AccessStatus::Cold
        }
    }

    fn access_storage(&mut self, address: Address, key: U256) -> AccessStatus {
        let value = self
            .accounts
            .entry(address)
            .or_default()
            .storage
            .entry(key)
            .or_default();
        let access_status = value.access_status;
        value.access_status = AccessStatus::Warm;
        access_status
    }
}