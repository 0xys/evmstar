use ethereum_types::{
    Address, U256
};

#[derive(Clone, Debug, Default)]
pub struct Journal {
    pub storage_log: Vec<StorageDelta>,
}

#[derive(Clone, Debug, Default)]
pub struct StorageDelta {
    pub address: Address,
    pub key: U256,
    pub previous: U256,
}

