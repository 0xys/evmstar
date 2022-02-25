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
use evmstar::interpreter::stack::Calldata;

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
        .append(OpCode::PUSH3)
        .append("123456")
        .append(OpCode::EXTCODEHASH)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN);
    
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
        .append(OpCode::PUSH3)
        .append("123456")
        .append(OpCode::EXTCODEHASH)
        .append(OpCode::PUSH3)
        .append("123456")
        .append(OpCode::EXTCODEHASH)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000aa").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(2721), output.gas_left);
}

#[test]
fn test_pre_eip1283_sstore() {
    for r in Revision::iter() {
        if r < Revision::Constantinople || r == Revision::Petersburg {
            test_sstore_of(r);
        }
    }
}

fn test_sstore_of(revision: Revision) {

    let original = 0x00;
    test_sstore_legacy_logic(hex!("60006000556000600055").to_vec(), 10012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60006000556001600055").to_vec(), 25012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60016000556000600055").to_vec(), 25012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60016000556002600055").to_vec(), 25012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60016000556001600055").to_vec(), 25012,     0, original, revision);

    let original = 0x01;
    test_sstore_legacy_logic(hex!("60006000556000600055").to_vec(), 10012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60006000556001600055").to_vec(), 25012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60006000556002600055").to_vec(), 25012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60026000556000600055").to_vec(), 10012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60026000556003600055").to_vec(), 10012,     0, original, revision);

    test_sstore_legacy_logic(hex!("60026000556001600055").to_vec(), 10012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60026000556002600055").to_vec(), 10012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60016000556000600055").to_vec(), 10012, 15000, original, revision);
    test_sstore_legacy_logic(hex!("60016000556002600055").to_vec(), 10012,     0, original, revision);
    test_sstore_legacy_logic(hex!("60016000556001600055").to_vec(), 10012,     0, original, revision);

    test_sstore_legacy_logic(hex!("600160005560006000556001600055").to_vec(), 45018, 15000, 0x00, revision);
    test_sstore_legacy_logic(hex!("600060005560016000556000600055").to_vec(), 30018, 30000, 0x01, revision);
}

fn test_sstore_legacy_logic(code: Vec<u8>, gas_used: i64, gas_refund: i64, original: usize, revision: Revision) {
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append(code.as_slice());
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, revision);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(gas_used, consumed_gas(output.gas_left));
    assert_eq!(gas_refund, output.gas_refund);
}

#[test]
fn test_sload() {
    let code = &hex!("60006000546000600054").to_vec();
    test_sload_logic(code, 112, Revision::Frontier);
    test_sload_logic(code, 112, Revision::Homestead);
    test_sload_logic(code, 412, Revision::Tangerine);
    test_sload_logic(code, 412, Revision::Spurious);
    test_sload_logic(code, 412, Revision::Byzantium);
    test_sload_logic(code, 412, Revision::Constantinople);
    test_sload_logic(code, 412, Revision::Petersburg);
    test_sload_logic(code, 1612, Revision::Istanbul);
    test_sload_logic(code, 2212, Revision::Berlin);
    test_sload_logic(code, 2212, Revision::London);
    test_sload_logic(code, 2212, Revision::Shanghai);
}

fn test_sload_logic(code: &Vec<u8>, gas_used: i64, revision: Revision) {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();
    let code = builder.append(code.as_slice());
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, revision);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(gas_used, consumed_gas(output.gas_left));
}

#[test]
fn test_calldatasize() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();
    let code = builder.append("3660005260206000f3");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();
    context.calldata = Calldata::from("ffff");

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Shanghai);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("0000000000000000000000000000000000000000000000000000000000000002"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(17, consumed_gas(output.gas_left));
}

#[test]
fn test_calldataload() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();
    let code = builder.append("60003560005260206000f3");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();
    context.calldata = Calldata::from("ffff");

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Shanghai);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("ffff000000000000000000000000000000000000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(21, consumed_gas(output.gas_left));
}

#[test]
fn test_calldataload_2() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 7feeee00000000000000000000000000000000000000000000000000000000000060005260003560105260206000f3
    let code = builder
        .append(OpCode::PUSH32)
        .append("eeee000000000000000000000000000000000000000000000000000000000000")
        .append(OpCode::PUSH1)
        .append(0)
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append(0)
        .append(OpCode::CALLDATALOAD)
        .append("60105260206000f3");

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();
    context.calldata = Calldata::from("ffff");

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Shanghai);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("eeee0000000000000000000000000000ffff0000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(33, consumed_gas(output.gas_left));
}

#[test]
fn test_calldatacopy() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 6020600060003760206000f3
    let code = builder
        .append("602060006000")
        .append(OpCode::CALLDATACOPY)
        .append("60206000")
        .append(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();
    context.calldata = Calldata::from("ffff");

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Berlin);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("ffff000000000000000000000000000000000000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(24, consumed_gas(output.gas_left));
}

#[test]
fn test_calldatacopy_2() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 7feeee0000000000000000000000000000000000000000000000000000000000006000526020600060103760206000f3
    let code = builder
        .append(OpCode::PUSH32)
        .append("eeee000000000000000000000000000000000000000000000000000000000000")
        .append(OpCode::PUSH1)
        .append(0)
        .append(OpCode::MSTORE)
        .append("602060006010")
        .append(OpCode::CALLDATACOPY)
        .append("60206000")
        .append(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();
    context.calldata = Calldata::from("ffff");

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Berlin);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("eeee0000000000000000000000000000ffff0000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(36, consumed_gas(output.gas_left));
}

#[test]
fn test_codesize() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 3860005260206000f3
    let code = builder
        .append(OpCode::CODESIZE)
        .append("6000")
        .append(OpCode::MSTORE)
        .append("60206000")
        .append(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Berlin);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("0000000000000000000000000000000000000000000000000000000000000009"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(17, consumed_gas(output.gas_left));
}

#[test]
fn test_codecopy() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 600c600060003960206000f3
    let code = builder
        .append("600c60006000")
        .append(OpCode::CODECOPY)
        .append("60206000")
        .append(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Berlin);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("600c600060003960206000f30000000000000000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(24, consumed_gas(output.gas_left));
}

#[test]
fn test_codecopy_out_of_bounds() {
    let host = StatefulHost::new_with(get_default_context());

    let mut builder = Code::builder();

    // 6040600060003960406000f3
    let code = builder
        .append(OpCode::PUSH1)
        .append(64) // out of bounds
        .append("60006000")
        .append(OpCode::CODECOPY)
        .append("60406000")
        .append(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Berlin);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("6040600060003960406000f300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(30, consumed_gas(output.gas_left));
}

fn default_contract() -> Account {
    let mut account = Account::default();
    account.code = Bytes::from(Vec::from(hex!("aabbccdd")));
    account.code_hash = U256::from(0x0011eeffu32);
    account
}

fn default_contract_address() -> Address {
    Address::from_low_u64_be(0x11223344)
}

fn test_extcodesize_logic(gas_used: i64, revision: Revision){
    let mut host = StatefulHost::new_with(get_default_context());
    host.add_account(default_contract_address(), default_contract());

    let mut builder = Code::builder();

    let code = builder
        // first access
        .append(OpCode::PUSH4)
        .append("11223344") // contract address
        .append(OpCode::EXTCODESIZE)

        // second access
        .append(OpCode::PUSH4)
        .append("11223344") // contract address
        .append(OpCode::EXTCODESIZE)
        .append("6000")
        .append(OpCode::MSTORE)

        .append("60206000")
        .append(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, revision);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("0000000000000000000000000000000000000000000000000000000000000004"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(gas_used, consumed_gas(output.gas_left));
}

#[test]
fn test_extcodesize(){
    for revision in Revision::iter() {
        if revision < Revision::Tangerine {
            test_extcodesize_logic(61, revision);
            continue;
        }else if revision < Revision::Berlin {
            test_extcodesize_logic(1421, revision);
            continue;
        }else{
            test_extcodesize_logic(2721, revision);
            continue;
        }
    }
}

fn test_extcodecopy_logic(gas_used: i64, revision: Revision) {
    let mut host = StatefulHost::new_with(get_default_context());
    host.add_account(default_contract_address(), default_contract());

    let mut builder = Code::builder();

    let code = builder
        // first access
        .append(OpCode::PUSH4)
        .append("11223344") // contract address
        .append(OpCode::EXTCODESIZE)

        // second access
        .append("600460006000")
        .append(OpCode::PUSH4)
        .append("11223344") // contract address
        .append(OpCode::EXTCODECOPY)

        .append("60206000")
        .append(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, revision);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("aabbccdd00000000000000000000000000000000000000000000000000000000"));
    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(bytes), output.data);
    assert_eq!(gas_used, consumed_gas(output.gas_left));
}

#[test]
fn test_extcodecopy() {
    for revision in Revision::iter() {
        if revision < Revision::Tangerine {
            test_extcodecopy_logic(21 + 20 + 20 + 6, revision); // push*7 + EXTCODESIZE + EXTCODECOPY + memory op
            continue;
        }else if revision < Revision::Berlin {
            test_extcodecopy_logic(21 + 700 + 700 + 6, revision);
            continue;
        }else{
            test_extcodecopy_logic(21 + 2600 + 100 + 6, revision);
            continue;
        }
    }
}


fn test_extcodehash_logic(gas_used: i64, revision: Revision) {
    let mut host = StatefulHost::new_with(get_default_context());
    host.add_account(default_contract_address(), default_contract());

    let mut builder = Code::builder();

    let code = builder
        // first access
        .append(OpCode::PUSH4)
        .append("11223344") // contract address
        .append(OpCode::EXTCODESIZE)

        // second access
        .append(OpCode::PUSH4)
        .append("11223344") // contract address
        .append(OpCode::EXTCODEHASH)
        .append("6000")
        .append(OpCode::MSTORE)

        .append("60206000")
        .append(OpCode::RETURN);

    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, revision);
    let output = executor.execute_raw_with(context);

    let bytes = Vec::from(hex!("000000000000000000000000000000000000000000000000000000000011eeff"));
    if revision < Revision::Constantinople {
        assert_eq!(StatusCode::Failure(FailureKind::InvalidInstruction), output.status_code); // extcodehash is added on Constantinople by EIP-1013
    }else{
        assert_eq!(StatusCode::Success, output.status_code);
        assert_eq!(Bytes::from(bytes), output.data);
        assert_eq!(gas_used, consumed_gas(output.gas_left));
    }    
}

#[test]
fn test_extcodehash() {
    for revision in Revision::iter() {
        if revision < Revision::Constantinople {
            test_extcodehash_logic(0, revision);
            continue;
        }
        if revision >= Revision::Berlin {
            test_extcodehash_logic(15 + 2600 + 100 + 6, revision);
            continue;
        }
        let extcodehash_cost = match revision {
            Revision::Istanbul => 700,
            Revision::Constantinople | Revision::Petersburg => 400,
            _ => panic!("undefined")
        };

        test_extcodehash_logic(15 + 700 + extcodehash_cost + 6, revision); // push*5 + EXTCODESIZE + EXTCODEHASH + memory op
    }
}