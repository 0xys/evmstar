use ethereum_types::{U256, Address};
use evmstar::tester::Evm;

use evmstar::model::{
    code::{
        Code, Append,
    },
    evmc::{
        StatusCode,
        TxContext,
    },
    revision::Revision,
};

fn default_address() -> Address { Address::from_low_u64_be(0xffffeeee) }

fn get_default_context() -> TxContext {
    TxContext {
        gas_price: U256::from(0x1234),
        origin: Address::from_low_u64_be(0x1234),
        coinbase: Address::from_low_u64_be(0xabcd),
        block_number: 0x1111,
        block_timestamp: 0x2222,
        gas_limit: 0x3333,
        base_fee: U256::from(0x4444),
        chain_id: U256::from(0x01),
        difficulty: U256::from(0x5555),
    }
}

fn sstore_eip2929(code: &str, gas_used: i64, gas_refund: i64, warm: bool, original: usize) {
    let code = Code::builder().append(code).clone();
    let gas_limit = i64::max_value();

    let mut tester = Evm::new_with(get_default_context());
    tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .with_storage(default_address(), U256::zero(), U256::from(original));
    
    if warm {
        tester.with_storage_always_warm();
    }

    let result = tester.run_code_as(code, Revision::Berlin);
    
    result.expect_status(StatusCode::Success)
        .expect_output("")
        .expect_gas(gas_used)
        .expect_gas_refund(gas_refund);
}

/// defined in https://eips.ethereum.org/EIPS/eip-3529
#[test]
fn test_eip2929_1(){
    let code = "60006000556000600055";
    let gas_used = 212;
    let gas_refund = 0;
    let original = 0x00;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_2(){
    let code = "60006000556001600055";
    let gas_used = 20112;
    let gas_refund = 0;
    let original = 0x00;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_3(){
    let code = "60016000556000600055";
    let gas_used = 20112;
    let gas_refund = 19900;
    let original = 0x00;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_4(){
    let code = "60016000556002600055";
    let gas_used = 20112;
    let gas_refund = 0;
    let original = 0x00;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_5(){
    let code = "60016000556001600055";
    let gas_used = 20112;
    let gas_refund = 0;
    let original = 0x00;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_6(){
    let code = "60006000556000600055";
    let gas_used = 3012;
    let gas_refund = 15000;
    let original = 0x01;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_7(){
    let code = "60006000556001600055";
    let gas_used = 3012;
    let gas_refund = 2800;
    let original = 0x01;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_8(){
    let code = "60006000556002600055";
    let gas_used = 3012;
    let gas_refund = 0;
    let original = 0x01;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_9(){
    let code = "60026000556000600055";
    let gas_used = 3012;
    let gas_refund = 15000;
    let original = 0x01;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_10(){
    let code = "60026000556003600055";
    let gas_used = 3012;
    let gas_refund = 0;
    let original = 0x01;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_11(){
    let code = "60026000556001600055";
    let gas_used = 3012;
    let gas_refund = 2800;
    let original = 0x01;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_12(){
    let code = "60026000556002600055";
    let gas_used = 3012;
    let gas_refund = 0;
    let original = 0x01;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_13(){
    let code = "60016000556000600055";
    let gas_used = 3012;
    let gas_refund = 15000;
    let original = 0x01;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_14(){
    let code = "60016000556002600055";
    let gas_used = 3012;
    let gas_refund = 0;
    let original = 0x01;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_15(){
    let code = "60016000556001600055";
    let gas_used = 212;
    let gas_refund = 0;
    let original = 0x01;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_16(){
    let code = "600160005560006000556001600055";
    let gas_used = 40118;
    let gas_refund = 19900;
    let original = 0x00;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_17(){
    let code = "600060005560016000556000600055";
    let gas_used = 5918;
    let gas_refund = 17800;
    let original = 0x01;
    sstore_eip2929(code, gas_used, gas_refund, true, original);
    sstore_eip2929(code, gas_used + 2100, gas_refund, false, original);
}