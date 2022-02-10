pub mod i256;

use ethereum_types::{U256, H256, Address};


pub(crate) fn u256_to_address(v: U256) -> Address {
    H256(v.into()).into()
}

#[allow(dead_code)]
pub(crate) fn address_to_u256(v: Address) -> U256 {
    U256::from_big_endian(&v.0)
}