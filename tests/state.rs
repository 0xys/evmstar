use bytes::Bytes;
use ethereum_types::{U256, Address};
use hex_literal::hex;

use evmstar::host::stateful::{
    StatefulHost, Account,
};
use evmstar::executor::{
    callstack::CallContext,
    executor::Executor,
};
#[allow(unused_imports)]
use evmstar::model::{
    code::{Code},
    opcode::OpCode,
    evmc::{
        StatusCode, FailureKind,
        TxContext,
    },
    revision::Revision,
};

use hex::{decode};

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

#[test]
fn test_extcodehash_cold() {
    let mut host = StatefulHost::new_with(get_default_context());
    let address = Address::from_low_u64_be(0x123456);
    let account = Account {
        nonce: 0,
        code: Bytes::default(),
        code_hash: U256::from(0xaa),
        balance: U256::from(0),
        storage: Default::default()
    };
    host.add_account(address, account);

    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH3)
        .append(&[0x12, 0x34, 0x56])
        .append_opcode(OpCode::EXTCODEHASH)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000aa").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(2618), output.gas_left);
}

#[test]
fn test_extcodehash_warm() {
    let mut host = StatefulHost::new_with(get_default_context());
    let address = Address::from_low_u64_be(0x123456);
    let account = Account {
        nonce: 0,
        code: Bytes::default(),
        code_hash: U256::from(0xaa),
        balance: U256::from(0),
        storage: Default::default()
    };
    host.add_account(address, account);

    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH3)
        .append(&[0x12, 0x34, 0x56])
        .append_opcode(OpCode::EXTCODEHASH)
        .append_opcode(OpCode::PUSH3)
        .append(&[0x12, 0x34, 0x56])
        .append_opcode(OpCode::EXTCODEHASH)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append(&[0x20])
        .append_opcode(OpCode::PUSH1)
        .append(&[0x00])
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000aa").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(2721), output.gas_left);
}

#[test]
/// test case from: https://eips.ethereum.org/EIPS/eip-1283
fn test_eip1283_1(){// 1
    let host = StatefulHost::new_with(get_default_context());
    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let mut builder = Code::builder();

    let code = builder.append(&hex!("60006000556000600055"));
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(412, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_2(){// 2
    let host = StatefulHost::new_with(get_default_context());
    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let mut builder = Code::builder();

    let code = builder.append(&hex!("60006000556001600055"));
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_3(){// 3
    let host = StatefulHost::new_with(get_default_context());
    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let mut builder = Code::builder();

    let code = builder.append(&hex!("60016000556000600055"));
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_4(){// 4
    let host = StatefulHost::new_with(get_default_context());
    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let mut builder = Code::builder();

    let code = builder.append(&hex!("60016000556002600055"));
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_5(){// 5
    let host = StatefulHost::new_with(get_default_context());
    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let mut builder = Code::builder();

    let code = builder.append(&hex!("60016000556001600055"));
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_6(){// 6
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x01));
    
    let mut builder = Code::builder();
    let code = builder.append(&hex!("60006000556000600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_7(){// 7
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append(&hex!("60006000556001600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_8(){// 8
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append(&hex!("60006000556002600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_9(){// 9
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append(&hex!("60026000556000600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_10(){// 10
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append(&hex!("60026000556003600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_11(){// 11
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append(&hex!("60026000556001600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_12(){// 12
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append(&hex!("60026000556002600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_13(){// 13
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append(&hex!("60016000556000600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_14(){// 14
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append(&hex!("60016000556002600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_15(){// 15
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append(&hex!("60016000556001600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(412, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_16(){// 16
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x00));

    let mut builder = Code::builder();
    let code = builder.append(&hex!("600160005560006000556001600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(40218, consumed_gas(output.gas_left));
}

#[test]
fn test_eip1283_17(){// 17
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append(&hex!("600060005560016000556000600055"));
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(10218, consumed_gas(output.gas_left));
}