use ethereum_types::{U256, Address};

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
use evmstar::tester::EvmTester;

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

fn test_common_with_original(code: &str, consumed_gas: i64, gas_refund: i64, original: i32) {
    let mut builder = Code::builder();
    let code = builder.append(code).clone();
    let gas_limit = i64::max_value();

    let mut tester = EvmTester::new_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .with_storage(default_address(), U256::zero(), U256::from(original))
        .with_to(default_address())
        .run_code_as(code, Revision::Istanbul);
    
    result.expect_status(StatusCode::Success)
        .expect_output("")
        .expect_gas(consumed_gas)
        .expect_gas_refund(gas_refund);
}

/// https://eips.ethereum.org/EIPS/eip-2200
#[test]
fn test_eip2200_1(){
    let original = 0x00;
    test_common_with_original("60006000556000600055", 1612, 0, original);
}

#[test]
fn test_eip2200_2(){
    let original = 0x00;
    test_common_with_original("60006000556001600055", 20812, 0, original);
}

#[test]
fn test_eip2200_3(){
    let original = 0x00;
    test_common_with_original("60016000556000600055", 20812, 19200, original);
}

#[test]
fn test_eip2200_4(){
    let original = 0x00;
    test_common_with_original("60016000556002600055", 20812, 0, original);
}

#[test]
fn test_eip2200_5(){
    let original = 0x00;
    test_common_with_original("60016000556001600055", 20812, 0, original);
}

#[test]
fn test_eip2200_6(){
    let original = 0x01;
    test_common_with_original("60006000556000600055", 5812, 15000, original);
}

#[test]
fn test_eip2200_7(){
    let original = 0x01;
    test_common_with_original("60006000556001600055", 5812, 4200, original);
}

#[test]
fn test_eip2200_8(){
    let original = 0x01;
    test_common_with_original("60006000556002600055", 5812, 0, original);
}

#[test]
fn test_eip2200_9(){
    let original = 0x01;
    test_common_with_original("60026000556000600055", 5812, 15000, original);
}

#[test]
fn test_eip2200_10(){
    let original = 0x01;
    test_common_with_original("60026000556003600055", 5812, 0, original);
}

#[test]
fn test_eip2200_11(){
    let original = 0x01;
    test_common_with_original("60026000556001600055", 5812, 4200, original);
}

#[test]
fn test_eip2200_12(){
    let original = 0x01;
    test_common_with_original("60026000556002600055", 5812, 0, original);
}

#[test]
fn test_eip2200_13(){
    let original = 0x01;
    test_common_with_original("60016000556000600055", 5812, 15000, original);
}

#[test]
fn test_eip2200_14(){
    let original = 0x01;
    test_common_with_original("60016000556002600055", 5812, 0, original);
}

#[test]
fn test_eip2200_15(){
    let original = 0x01;
    test_common_with_original("60016000556001600055", 1612, 0, original);
}

#[test]
fn test_eip2200_16(){
    let original = 0x00;
    test_common_with_original("600160005560006000556001600055", 40818, 19200, original);
}

#[test]
fn test_eip2200_17(){
    let original = 0x01;
    test_common_with_original("600060005560016000556000600055", 10818, 19200, original);
}