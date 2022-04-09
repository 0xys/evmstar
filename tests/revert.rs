use ethereum_types::{U256, Address};
use evmstar::emulator::EvmEmulator;
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
    
    let mut tester = EvmEmulator::new_stateful_with(get_default_context());
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
    
    let mut tester = EvmEmulator::new_stateful_with(get_default_context());
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
fn address_value(num: u64) -> U256 {
    U256::from(0xeeeeee00 + num)
}
fn call_code_and_add(address_u64: u64) -> Code {
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
        code.append_code(&mut call_code_and_add(i as u64));
    }
    let code = code
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::RETURN)
        .clone();

    let mut tester = EvmEmulator::new_stateful_with(get_default_context());
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

fn call_code(address_u64: u64, value: i32) -> Code {
    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_size
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_offset
        .append(OpCode::PUSH1)
        .append(0x00)   // args_size
        .append(OpCode::PUSH1)
        .append(0x00)   // args_offset
        .append(OpCode::PUSH32)
        .append(U256::from(value))   // value
        .append(OpCode::PUSH20)
        .append(address(address_u64))   // address
        .append(OpCode::GAS)    // 2
        .append(OpCode::CALL)
        .clone();
    code
}

fn return_contract() -> Code {
    let code = Code::builder()
        .append(OpCode::ADDRESS)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::SSTORE)
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::RETURN)
        .clone();
    code
}
fn revert_contract() -> Code {
    let code = Code::builder()
        .append(OpCode::ADDRESS)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::SSTORE)
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::REVERT)
        .clone();
    code
}
fn callreturn_contract(called_address: u64) -> Code {
    let code = Code::builder()
        .append(OpCode::ADDRESS)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::SSTORE)
        .append_code(&mut call_code(called_address, 0))
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::RETURN)
        .clone();
    code
}
fn callrevert_contract(called_address: u64) -> Code {
    let code = Code::builder()
        .append(OpCode::ADDRESS)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::SSTORE)
        .append_code(&mut call_code(called_address, 0))
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::REVERT)
        .clone();
    code
}

#[test]
fn test_stackitem_after_revert() {
    let mut tester = EvmEmulator::new_stateful_with(get_default_context());
    tester.with_default_gas()
        .with_contract_deployed2(address(1), return_contract(), U256::zero())
        .with_contract_deployed2(address(2), revert_contract(), U256::zero());
    
    let code = Code::builder()
        .append_code(&mut call_code(1, 0)) // success = push 1
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::MSTORE)

        .append_code(&mut call_code(2, 0)) // revert = push 0
        .append(OpCode::ISZERO)
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::MSTORE)

        .append(OpCode::PUSH1)
        .append(0x40)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::RETURN)
        .clone();

    let result = tester.run_code(code);
    result.expect_status(StatusCode::Success)
        .expect_output("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001"); // 0x01 and 0x00
}

#[test]
fn test_revert_deep() {
    let mut tester = EvmEmulator::new_stateful_with(get_default_context());
    tester.with_default_gas()
        // [call, call, call, return, return, return]
        //      | 
        // call + ---> + 1
        //      |      |
        //      | call + ---> + 2
        //      |             |
        //      |        call + ---> + 3
        //      |                    |
        //      |             + <--- + return
        //      |             |
        //      |      + <--- + return
        //      |      |
        //      + <--- + return
        .with_contract_deployed2(address(1), callreturn_contract(2), U256::zero())
        .with_contract_deployed2(address(2), callreturn_contract(3), U256::zero())
        .with_contract_deployed2(address(3), return_contract(), U256::zero())

        // [call, call, call, revert, return, return]
        //      | 
        // call + ---> + 4
        //      |      |
        //      | call + ---> + 5
        //      |             |
        //      |        call + ---> + 6
        //      |                    |
        //      |             + <--- + revert
        //      |             |
        //      |      + <--- + return
        //      |      |
        //      + <--- + return
        .with_contract_deployed2(address(4), callreturn_contract(5), U256::zero())
        .with_contract_deployed2(address(5), callreturn_contract(6), U256::zero())
        .with_contract_deployed2(address(6), revert_contract(), U256::zero())

        // [call, call, call, return, revert, return]
        //      | 
        // call + ---> + 7
        //      |      |
        //      | call + ---> + 8
        //      |             |
        //      |        call + ---> + 9
        //      |                    |
        //      |             + <--- + return
        //      |             |
        //      |      + <--- + revert
        //      |      |
        //      + <--- + return
        .with_contract_deployed2(address(7), callreturn_contract(8), U256::zero())
        .with_contract_deployed2(address(8), callrevert_contract(9), U256::zero())
        .with_contract_deployed2(address(9), return_contract(), U256::zero())

        // [call, call, call, return, return, revert]
        //      | 
        // call + ---> + 10
        //      |      |
        //      | call + ---> + 11
        //      |             |
        //      |        call + ---> + 12
        //      |                    |
        //      |             + <--- + return
        //      |             |
        //      |      + <--- + return
        //      |      |
        //      + <--- + revert
        //      | 
        .with_contract_deployed2(address(10), callrevert_contract(11), U256::zero())
        .with_contract_deployed2(address(11), callreturn_contract(12), U256::zero())
        .with_contract_deployed2(address(12), return_contract(), U256::zero());

    let code = Code::builder()
        .append_code(&mut call_code(1, 0))
        .append_code(&mut call_code(4, 0))
        .append_code(&mut call_code(7, 0))
        .append_code(&mut call_code(10, 0))
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::RETURN)
        .clone();

    let result = tester.run_code(code);
    result.expect_status(StatusCode::Success)
        .expect_storage(address(1), U256::zero(), address_value(1))
        .expect_storage(address(2), U256::zero(), address_value(2))
        .expect_storage(address(3), U256::zero(), address_value(3))

        .expect_storage(address(4), U256::zero(), address_value(4))
        .expect_storage(address(5), U256::zero(), address_value(5))
        .expect_storage(address(6), U256::zero(), U256::zero())

        .expect_storage(address(7), U256::zero(), address_value(7))
        .expect_storage(address(8), U256::zero(), U256::zero())
        .expect_storage(address(9), U256::zero(), U256::zero())

        .expect_storage(address(10), U256::zero(), U256::zero())
        .expect_storage(address(11), U256::zero(), U256::zero())
        .expect_storage(address(12), U256::zero(), U256::zero());
}

#[test]
fn test_revert_balance_transfer() {
    let to = address(0xeeee);
    let ret_address = 0x00;
    let rev_address = 0x01;
    let value = 0x01;

    let mut tester = EvmEmulator::new_stateful_with(get_default_context());
    tester.with_default_gas()
        .with_to(to)
        .with_account(to, U256::from(0xffff))
        .with_contract_deployed2(address(ret_address), return_contract(), U256::zero())
        .with_contract_deployed2(address(rev_address), revert_contract(), U256::zero())
        ;
    
    let code = Code::builder()
        .append_code(&mut call_code(ret_address, value))
        .append_code(&mut call_code(rev_address, value))
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::RETURN)
        .clone();

    let result = tester.run_code(code);
    result.expect_status(StatusCode::Success)
        .expect_balance(to, U256::from(0xffff - 1))
        .expect_balance(address(ret_address), U256::from(value))    // transfered
        .expect_balance(address(rev_address), U256::zero())     // transfer reverted
        .expect_storage(address(ret_address), U256::zero(), address_value(ret_address))
        .expect_storage(address(rev_address), U256::zero(), U256::zero())
        ;
}

// #[test]
// fn test_revert_gas_refund() {
//     // TODO
//     assert!(false);
// }

// #[test]
// fn test_reverted_create() {
//     // TODO
//     assert!(false);
// }
// #[test]
// fn test_reverted_create2() {
//     // TODO
//     assert!(false);
// }