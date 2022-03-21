use std::rc::Rc;
use std::cell::RefCell;

use bytes::Bytes;
use ethereum_types::{U256, Address};
use evmstar::executor::callstack::CallScope;
use hex::decode;

use evmstar::host::stateful::{
    StatefulHost,
};
use evmstar::executor::{
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
fn consumed_gas(amount: i64, gas_limit: i64) -> i64 {
    gas_limit - amount
}

fn get_default_context() -> TxContext {
    TxContext {
        gas_price: U256::from(0x1234),
        origin: Address::from_low_u64_be(0x1234),
        coinbase: Address::from_low_u64_be(0xabcd),
        block_number: 0x1111,
        block_timestamp: 0x2222,
        gas_limit: 100_000,
        base_fee: U256::from(0x4444),
        chain_id: U256::from(0x01),
        difficulty: U256::from(0x5555),
    }
}

#[test]
fn test_revert_one_level() {
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));

    let mut builder = Code::builder();
    let code = builder
        .append(OpCode::PUSH1)  // 3
        .append(0xdd)           // data
        .append(OpCode::PUSH1)  // 3
        .append(0x00)           // offset
        .append(OpCode::SSTORE) // 20000 + 2100
        .append("60aa60005260206000") // 3*4 + 6 = 18
        .append(OpCode::REVERT) // 0
        .clone(); // = 22124
    
    let gas_limit = 100_000;
    let mut scope = CallScope::default();
    scope.code = code;
    scope.to = default_address();
    scope.gas_limit = gas_limit;
    scope.gas_left = gas_limit;
    
    let mut executor = Executor::new_with_tracing(host.clone());
    let output = executor.execute_raw_with(scope);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000aa").unwrap();

    assert_eq!(StatusCode::Failure(FailureKind::Revert), output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(22124, consumed_gas(output.gas_left, gas_limit));

    let value = (*host).borrow().debug_get_storage(default_address(), U256::from(0x01));
    assert_eq!(U256::from(0x00), value);    // set value is reverted
}