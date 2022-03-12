use ethereum_types::{Address, U256};
use bytes::Bytes;
use std::{
    collections::HashMap,
    sync::Mutex,
};

use crate::executor::journal::{Journal, Snapshot};
use crate::host::Host;
use crate::model::code::Code;
use crate::model::evmc::{
    Message, Output, TxContext, AccessStatus, StatusCode, StorageStatus, StorageStatusKind,
};
use hex_literal::hex;
use hex::decode;

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
    pub original_value: U256,
    pub current_value: U256,
    pub access_status: AccessStatus,
    pub dirty: bool,
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
    is_always_warm: bool,
    journal: Journal,
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
            recorded: Mutex::default(),
            is_always_warm: false,
            journal: Journal::default(),
        }
    }

    pub fn new_with(context: TxContext) -> Self {
        Self {
            context: context,
            accounts: Default::default(),
            recorded: Mutex::default(),
            is_always_warm: false,
            journal: Journal::default(),
        }
    }
}

impl StatefulHost {
    pub fn add_account(&mut self, address: Address, account: Account) {
        self.accounts.insert(address, account);
    }
    pub fn debug_get_storage(&self, address: Address, key: U256) -> U256 {
        self.accounts
            .get(&address)
            .and_then(|account| account.storage.get(&key).map(|value| value.current_value))
            .unwrap_or_else(U256::zero)
    }
    pub fn debug_set_storage(&mut self, address: Address, key: U256, new_value: U256) {
        let value = self
            .accounts
            .entry(address)
            .or_default()
            .storage
            .entry(key)
            .or_default();

        value.original_value = new_value;
        value.current_value = new_value;
    }
    pub fn debug_set_storage_as_warm(&mut self) {
        self.is_always_warm = true;
    }
    pub fn debug_deploy_contract(&mut self, address_hex: &str, code: Code, balance: U256) {
        let mut dst = [0u8; 20];
        let hex = decode(address_hex).unwrap();
        for i in 0..hex.len() {
            dst[hex.len() - 1 - i] = hex[hex.len() - 1 - i];
        }

        let account = Account {
            balance,
            code: code.0.into(),
            code_hash: U256::from(0x123456),
            nonce: 0,
            storage: Default::default(),
        };
        let address = Address::from_slice(&dst);
        self.accounts.insert(address, account);
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
            .and_then(|account| account.storage.get(&key).map(|value| value.current_value))
            .unwrap_or_else(U256::zero)
    }

    fn set_storage(&mut self, address: Address, key: U256, new_value: U256) -> StorageStatus {
        let mut record = self.recorded.lock().unwrap();
        record.record_account_access(address);

        // Get the reference to the old value.
        // This will create the account in case it was not present.
        // This is convenient for unit testing and standalone EVM execution to preserve the
        // storage values after the execution terminates.
        let value = self
            .accounts
            .entry(address)
            .or_default()
            .storage
            .entry(key)
            .or_default();
        
        self.journal.record_storage(address, key, value.current_value);

        // Follow https://eips.ethereum.org/EIPS/eip-1283 specification.
        if value.current_value == new_value {
            return StorageStatus{
                original: value.original_value,
                current: value.current_value,
                kind: StorageStatusKind::Unchanged
            }
        }

        let kind = if value.dirty {
            StorageStatusKind::ModifiedAgain
        }else{
            value.dirty = true;
            if value.current_value.is_zero() {
                StorageStatusKind::Added
            }else if new_value.is_zero() {
                StorageStatusKind::Deleted
            }else{
                StorageStatusKind::Modified
            }
        };

        let current_value_before_set = value.current_value;
        value.current_value = new_value;

        return StorageStatus {
            original: value.original_value,
            current: current_value_before_set,
            kind: kind
        }
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
            size: 0,
            gas_refund: 0,
            effective_gas_refund: 0,
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
        if self.is_always_warm {
            return AccessStatus::Warm;
        }

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
        if self.is_always_warm {
            return AccessStatus::Warm;
        }
        
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

    fn get_blockhash(&self, height: usize) -> U256 {
        U256::from(height)
    }
    fn get_code(&self, address: Address, offset: usize, size: usize) -> Bytes {
        let mut record = self.recorded.lock().unwrap();
        record.record_account_access(address);

        self.accounts
            .get(&address)
            .map(|account| account.code.clone())
            .unwrap_or_else(Bytes::default)
    }
    fn add_balance(&mut self, address: Address, amount: U256){
        let account = self
            .accounts
            .entry(address)
            .or_default();
        
        account.balance += amount;
    }
    fn subtract_balance(&mut self, address: Address, amount: U256){
        let account = self
            .accounts
            .entry(address)
            .or_default();
        
        account.balance -= amount;
    }
    fn rollback(&mut self, snapshot: Snapshot) {
        let length = self.journal.storage_log.len();
        for _ in 0..length - 1 - snapshot {
            if let Some(delta) = self.journal.storage_log.pop() {
                self.force_set_storage(delta.address, delta.key, delta.previous);
            }
        }
    }
    fn force_set_storage(&mut self, address: Address, key: U256, new_value: U256) {
        let value = self
            .accounts
            .entry(address)
            .or_default()
            .storage
            .entry(key)
            .or_default();
        value.current_value = new_value;
    }
}