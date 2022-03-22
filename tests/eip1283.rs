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

fn test_common(code: &str, consumed_gas: i64, gas_refund: i64) {
    let mut builder = Code::builder();
    let code = builder.append(code).clone();
    let gas_limit = i64::max_value();

    let mut tester = EvmTester::new_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .run_code_as(code, Revision::Constantinople);
    
    result.expect_status(StatusCode::Success)
        .expect_output("")
        .expect_gas(consumed_gas)
        .expect_gas_refund(gas_refund);
}

fn test_common_with_initial_state(code: &str, consumed_gas: i64, gas_refund: i64) {
    let mut builder = Code::builder();
    let code = builder.append(code).clone();
    let gas_limit = i64::max_value();

    let mut tester = EvmTester::new_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .with_storage(default_address(), U256::zero(), U256::from(0x01))
        .with_to(default_address())
        .run_code_as(code, Revision::Constantinople);
    
    result.expect_status(StatusCode::Success)
        .expect_output("")
        .expect_gas(consumed_gas)
        .expect_gas_refund(gas_refund);
}

#[test]
/// test case from: https://eips.ethereum.org/EIPS/eip-1283
fn test_eip1283_1(){// 1
    test_common("60006000556000600055", 412, 0);
}

#[test]
fn test_eip1283_2(){// 2
    test_common("60006000556001600055", 20212, 0);
}

#[test]
fn test_eip1283_3(){// 3
    test_common("60016000556000600055", 20212, 19800);
}

#[test]
fn test_eip1283_4(){// 4
    test_common("60016000556002600055", 20212, 0);
}

#[test]
fn test_eip1283_5(){// 5
    test_common("60016000556001600055", 20212, 0);
}

#[test]
fn test_eip1283_6(){// 6
    test_common_with_initial_state("60006000556000600055", 5212, 15000);
}

#[test]
fn test_eip1283_7(){// 7
    test_common_with_initial_state("60006000556001600055", 5212, 4800);
}

#[test]
fn test_eip1283_8(){// 8
    test_common_with_initial_state("60006000556002600055", 5212, 0);
}

#[test]
fn test_eip1283_9(){// 9
    test_common_with_initial_state("60026000556000600055", 5212, 15000);
}

#[test]
fn test_eip1283_10(){// 10
    test_common_with_initial_state("60026000556003600055", 5212, 0);
}

#[test]
fn test_eip1283_11(){// 11
    test_common_with_initial_state("60026000556001600055", 5212, 4800);
}

#[test]
fn test_eip1283_12(){// 12
    test_common_with_initial_state("60026000556002600055", 5212, 0);
}

#[test]
fn test_eip1283_13(){// 13
    test_common_with_initial_state("60016000556000600055", 5212, 15000);
}

#[test]
fn test_eip1283_14(){// 14
    test_common_with_initial_state("60016000556002600055", 5212, 0);
}

#[test]
fn test_eip1283_15(){// 15
    test_common_with_initial_state("60016000556001600055", 412, 0);
}

#[test]
fn test_eip1283_16(){// 16
    let code = "600160005560006000556001600055";
    let mut builder = Code::builder();
    let code = builder.append(code).clone();
    let gas_limit = i64::max_value();

    let mut tester = EvmTester::new_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .with_storage(default_address(), U256::zero(), U256::from(0x00))
        .with_to(default_address())
        .run_code_as(code, Revision::Constantinople);
    
    result.expect_status(StatusCode::Success)
        .expect_output("")
        .expect_gas(40218)
        .expect_gas_refund(19800);
}

#[test]
fn test_eip1283_17(){// 17
    test_common_with_initial_state("600060005560016000556000600055", 10218, 19800);
}