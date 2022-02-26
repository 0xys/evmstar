use bytes::Bytes;
use ethereum_types::{U256, Address};
use hex_literal::hex;

use evmstar::host::stateful::{
    StatefulHost,
};
use evmstar::executor::{
    callstack::CallScope,
    executor::Executor,
};
#[allow(unused_imports)]
use evmstar::model::{
    code::{
        Code, Append,
    },
    opcode::OpCode,
    evmc::{
        StatusCode, FailureKind,
        TxContext,
    },
    revision::Revision,
};

fn default_address() -> Address { Address::from_low_u64_be(0xffffeeee) }

fn consumed_gas(amount: i64) -> i64 {
    i64::max_value() - amount
}
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

fn sstore_eip2929(code: Vec<u8>, gas_used: i64, gas_refund: i64, warm: bool, original: usize) {
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));
    if warm {
        host.debug_set_storage_as_warm();
    }

    let mut builder = Code::builder();
    let code = builder.append(code.as_slice());
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Berlin);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(gas_used, consumed_gas(output.gas_left));
    assert_eq!(gas_refund, output.gas_refund);
}

/// defined in https://eips.ethereum.org/EIPS/eip-3529
#[test]
fn test_eip2929_1(){
    let code = hex!("60006000556000600055");
    let gas_used = 212;
    let gas_refund = 0;
    let original = 0x00;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_2(){
    let code = hex!("60006000556001600055");
    let gas_used = 20112;
    let gas_refund = 0;
    let original = 0x00;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_3(){
    let code = hex!("60016000556000600055");
    let gas_used = 20112;
    let gas_refund = 19900;
    let original = 0x00;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_4(){
    let code = hex!("60016000556002600055");
    let gas_used = 20112;
    let gas_refund = 0;
    let original = 0x00;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_5(){
    let code = hex!("60016000556001600055");
    let gas_used = 20112;
    let gas_refund = 0;
    let original = 0x00;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_6(){
    let code = hex!("60006000556000600055");
    let gas_used = 3012;
    let gas_refund = 15000;
    let original = 0x01;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_7(){
    let code = hex!("60006000556001600055");
    let gas_used = 3012;
    let gas_refund = 2800;
    let original = 0x01;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_8(){
    let code = hex!("60006000556002600055");
    let gas_used = 3012;
    let gas_refund = 0;
    let original = 0x01;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_9(){
    let code = hex!("60026000556000600055");
    let gas_used = 3012;
    let gas_refund = 15000;
    let original = 0x01;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_10(){
    let code = hex!("60026000556003600055");
    let gas_used = 3012;
    let gas_refund = 0;
    let original = 0x01;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_11(){
    let code = hex!("60026000556001600055");
    let gas_used = 3012;
    let gas_refund = 2800;
    let original = 0x01;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_12(){
    let code = hex!("60026000556002600055");
    let gas_used = 3012;
    let gas_refund = 0;
    let original = 0x01;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_13(){
    let code = hex!("60016000556000600055");
    let gas_used = 3012;
    let gas_refund = 15000;
    let original = 0x01;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_14(){
    let code = hex!("60016000556002600055");
    let gas_used = 3012;
    let gas_refund = 0;
    let original = 0x01;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_15(){
    let code = hex!("60016000556001600055");
    let gas_used = 212;
    let gas_refund = 0;
    let original = 0x01;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_16(){
    let code = hex!("600160005560006000556001600055");
    let gas_used = 40118;
    let gas_refund = 19900;
    let original = 0x00;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}

#[test]
fn test_eip2929_17(){
    let code = hex!("600060005560016000556000600055");
    let gas_used = 5918;
    let gas_refund = 17800;
    let original = 0x01;
    sstore_eip2929(code.to_vec(), gas_used, gas_refund, true, original);
    sstore_eip2929(code.to_vec(), gas_used + 2100, gas_refund, false, original);
}