use ethereum_types::{
    Address, U256
};

#[derive(Clone, Debug, Default)]
pub struct Journal {
    pub storage_log: Vec<StorageDelta>,
    pub balance_log: Vec<BalanceDelta>,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Snapshot {
    pub storage_snapshot: usize,
    pub balance_snapshot: usize,
}

impl Journal {
    pub fn record_storage(&mut self, address: Address, key: U256, value: U256) {
        let delta = StorageDelta {
            address,
            key,
            previous: value,
        };
        self.storage_log.push(delta);
    }

    pub fn record_balance_delta(&mut self, address: Address, sign: Sign, amount: U256) {
        let delta = BalanceDelta {
            address,
            sign,
            amount,
        };
        self.balance_log.push(delta);
    }
}

#[derive(Clone, Debug, Default)]
pub struct StorageDelta {
    pub address: Address,
    pub key: U256,
    pub previous: U256,
}

#[derive(Clone, Debug)]
pub enum Sign {
    Plus,
    Minus,
    Zero,
}
impl Default for Sign {
    fn default() -> Self {
        Self::Plus
    }
}

#[derive(Clone, Debug, Default)]
pub struct BalanceDelta {
    pub address: Address,
    pub sign: Sign,
    pub amount: U256,
}