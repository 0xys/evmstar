use ethereum_types::U256;

use crate::model::{
    evmc::{StatusCode, FailureKind},
    revision::Revision,
};

pub fn exp(base: &mut U256, power: &mut U256, gas_left: i64, revision: Revision) -> Result<(U256, i64), StatusCode> {
    if !power.is_zero() {
        let additional_gas = if revision >= Revision::Spurious {
            50
        } else {
            10
        } * (log2floor(*power) / 8 + 1);

        let gas_left = gas_left - additional_gas as i64;

        if gas_left < 0 {
            return Err(StatusCode::Failure(FailureKind::OutOfGas));
        }
    }

    let mut v = U256::one();

    while !power.is_zero() {
        if !(*power & U256::one()).is_zero() {
            v = v.overflowing_mul(*base).0;
        }
        *power >>= 1;
        *base = (*base).overflowing_mul(*base).0;
    }

    Ok((v, gas_left))
}

fn log2floor(value: U256) -> u64 {
    assert!(value != U256::zero());
    let mut l: u64 = 256;
    for i in 0..4 {
        let i = 3 - i;
        if value.0[i] == 0u64 {
            l -= 64;
        } else {
            l -= value.0[i].leading_zeros() as u64;
            if l == 0 {
                return l;
            } else {
                return l - 1;
            }
        }
    }
    l
}