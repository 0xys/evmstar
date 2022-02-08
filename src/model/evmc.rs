use ethereum_types::{Address, U256};
use bytes::Bytes;

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

/// https://evmc.ethereum.org/structevmc__result.html
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub status_code: StatusCode,
    pub gas_left: i64,
    pub data: Bytes,
    pub size: usize,
    pub create_address: Option<Address>,
}
impl Default for Output {
    fn default() -> Self {
        Output {
            gas_left: i64::max_value(),
            status_code: StatusCode::Success,
            create_address: None,
            data: Bytes::default(),
            size: 0
        }
    }
}
impl Output {
    pub fn default_failure() -> Self {
        Output {
            gas_left: i64::max_value(),
            status_code: StatusCode::Failure(FailureKind::Generic("default".to_owned())),
            create_address: None,
            data: Bytes::default(),
            size: 0
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
    StaticModeViolcation,
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
#[derive(Clone, Debug, PartialEq)]
pub enum AccessStatus {
    Cold,
    Warm,
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