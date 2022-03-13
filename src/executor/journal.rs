use ethereum_types::{
    Address, U256
};

#[derive(Clone, Debug, Default)]
pub struct Journal {
    pub storage_log: Vec<StorageDelta>,
}

pub type Snapshot = usize;

impl Journal {
    pub fn record_storage(&mut self, address: Address, key: U256, value: U256) {
        let delta = StorageDelta {
            address,
            key,
            previous: value,
        };
        self.storage_log.push(delta);
    }
}

#[derive(Clone, Debug, Default)]
pub struct StorageDelta {
    pub address: Address,
    pub key: U256,
    pub previous: U256,
}

