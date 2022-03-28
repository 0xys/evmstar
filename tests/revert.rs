use ethereum_types::{U256, Address};
use evmstar::tester::EvmTester;
use evmstar::model::{
    code::{
        Code, Append,
    },
    opcode::OpCode,
    evmc::{
        StatusCode, FailureKind,
        TxContext,
    }
};

fn default_address() -> Address { Address::from_low_u64_be(0xffffeeee) }

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
    let key = 0x01;

    let mut builder = Code::builder();
    let code = builder
        .append(OpCode::PUSH1)  // 3
        .append(0xdd)           // data
        .append(OpCode::PUSH1)  // 3
        .append(key)           // offset
        .append(OpCode::SSTORE) // 20000 + 2100
        .append("60aa60005260206000") // 3*4 + 6 = 18
        .append(OpCode::REVERT) // 0
        .clone(); // = 22124
    
    let gas_limit = 100_000;
    
    let mut tester = EvmTester::new_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .run_code(code);
    
    result.expect_status(StatusCode::Failure(FailureKind::Revert))
        .expect_output("00000000000000000000000000000000000000000000000000000000000000aa")
        .expect_gas(22124)
        .expect_storage(default_address(), U256::from(key), U256::from(0x00));
}

#[test]
fn test_revert_one_level_with_original() {
    let key = 0x01;

    let mut builder = Code::builder();
    let code = builder
        .append(OpCode::PUSH1)  // 3
        .append(0xdd)           // data
        .append(OpCode::PUSH1)  // 3
        .append(key)           // offset
        .append(OpCode::SSTORE) // 2900 + 2100
        .append("60aa60005260206000") // 3*4 + 6 = 18
        .append(OpCode::REVERT) // 0
        .clone(); // = 5024
    
    let gas_limit = 100_000;
    
    let mut tester = EvmTester::new_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .with_storage(default_address(), U256::from(key), U256::from(0xaa))
        .run_code(code);
    
    result.expect_status(StatusCode::Failure(FailureKind::Revert))
        .expect_output("00000000000000000000000000000000000000000000000000000000000000aa")
        .expect_gas(5024)
        .expect_storage(default_address(), U256::from(key), U256::from(0xaa));
}

fn scope_code(index: u64) -> Code {
    let mut sstore = Code::builder()
        .append(OpCode::PUSH32) // 3
        .append(U256::from(index))  // value
        .append(OpCode::PUSH1) // 3
        .append(0x00)   // key
        .append(OpCode::SSTORE) // 20000(init to non-zero) + 2100(cold)
        .clone();   // 22106
    
    let mut ret = Code::builder()
        .append(OpCode::PUSH32)
        .append(U256::from(index))
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(if index % 2 == 0 { OpCode::RETURN } else { OpCode::REVERT })
        .clone();
    
    let code = Code::builder()
        .append_code(&mut sstore)
        .append_code(&mut ret)
        .clone();
    code
}

fn address(num: u64) -> Address {
    Address::from_low_u64_be(0xeeeeee00 + num)
}
fn call_code(address_u64: u64) -> Code {
    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append(0x20)   // ret_size
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_offset
        .append(OpCode::PUSH1)
        .append(0x00)   // args_size
        .append(OpCode::PUSH1)
        .append(0x00)   // args_offset
        .append(OpCode::PUSH32)
        .append(U256::from(0x00))   // value
        .append(OpCode::PUSH20)
        .append(address(address_u64))   // address
        .append(OpCode::GAS)    // 2
        .append(OpCode::CALL)

        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::MLOAD)  // load from 'sum'
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::MLOAD)  // load from returned data
        .append(OpCode::ADD)    // add returned data to 'sum'

        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::MSTORE) // store to 'sum'
        .clone();
    code
}

#[test]
fn test_multiple_revert() {
    let mut code = Code::builder();

    let max = 12;

    for i in 0..max {
        code.append_code(&mut call_code(i as u64));
    }
    let code = code
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::RETURN)
        .clone();

    let mut tester = EvmTester::new_with(get_default_context());
    tester.with_default_gas();

    for i in 0..max {
        tester.with_contract_deployed2(address(i), scope_code(i), U256::zero());
    }
    let result = tester.run_code(code);

    result.expect_status(StatusCode::Success);
    for i in 0..max {
        let expected_value = 
            if i % 2 == 0 {
                U256::from(i)
            }else{
                U256::zero()
            };
        result.expect_storage(address(i), U256::zero(), expected_value);
    }

    result.expect_output("0000000000000000000000000000000000000000000000000000000000000042"); // 0x42 = 66 = 0 + 1 + 2 + ... + 11)
}