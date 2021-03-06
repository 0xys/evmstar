use std::str::FromStr;

use ethereum_types::{U256, Address};
use hex::decode;

use evmstar::emulator::{
    EvmEmulator,
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

fn get_address(address_hex: &str) -> Address {
    let mut dst = [0u8; 20];
    let hex = decode(address_hex).unwrap();
    for i in 0..hex.len() {
        dst[hex.len() - 1 - i] = hex[hex.len() - 1 - i];
    }
    let address = Address::from_slice(&dst);
    address
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
fn get_code_for_call(gas: i64, address: &str, value: usize, args_offset: u8, args_size: u8, ret_offset: u8, ret_size: u8) -> Code {
    let address = get_address(address);

    let mut builder = Code::builder();
    let code = builder
        .append(OpCode::PUSH1)
        .append(ret_size)
        .append(OpCode::PUSH1)
        .append(ret_offset)
        .append(OpCode::PUSH1)
        .append(args_size)
        .append(OpCode::PUSH1)
        .append(args_offset)
        .append(OpCode::PUSH32)
        .append(U256::from(value))
        .append(OpCode::PUSH20)
        .append(address)
        .append(OpCode::PUSH32)
        .append(U256::from(gas))
        .append(OpCode::CALL);
    code.clone()    // 3 * 7 + 2600 = 2621
}

#[test]
fn test_call() {
    let mut builder = Code::builder();
    let contract = builder
        .append(OpCode::PUSH1)  // 3
        .append(0x00)
        .append(OpCode::CALLDATALOAD)   // 3
        .append(OpCode::PUSH1)  // 3
        .append(2)
        .append(OpCode::ADD)    // 3
        .append(OpCode::PUSH1)  // 3
        .append(0x00)
        .append(OpCode::MSTORE) // 3 + 3 = 6
        .append(OpCode::PUSH1)  // 3
        .append(0x20)
        .append(OpCode::PUSH1)  // 3
        .append(0x00)
        .append(OpCode::RETURN) // 0
        .clone(); // = 27

    let mut builder = Code::builder();
    let contract_address = "cc000001"; // 0xcc00000100000000000000000000000000000000 
    let code = builder
        .append(OpCode::PUSH1)  // 3
        .append(0xa0)
        .append(OpCode::PUSH1)  // 3
        .append(0x00)
        .append(OpCode::MSTORE) // 3 + 3 = 6
        .append_code(&mut get_code_for_call(50_000, contract_address, 0, 0, 0x20, 0, 0x20))
            // call cost = 2621
            // execution cost = 27
            // = 2648
        .append(OpCode::PUSH1)  // 3
        .append(0x20)
        .append(OpCode::PUSH1)  // 3
        .append(0x00)
        .append(OpCode::RETURN) // 0
        .clone();
    
    let gas_limit = 100_000;

    let mut tester = EvmEmulator::new_stateful_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .with_contract_deployed(contract_address, contract, U256::zero())
        .run_code(code);
    
    result.expect_status(StatusCode::Success)
        .expect_output("00000000000000000000000000000000000000000000000000000000000000a2")
        .expect_gas(2666);
}

#[test]
fn test_remote_self_balance() {
    let mut builder = Code::builder();
    let contract = builder
        .append(OpCode::SELFBALANCE)    // get contract balance
        .append("6000")
        .append(OpCode::MSTORE)
        .append("60206000")
        .append(OpCode::RETURN)
        .clone();   // = 20
    
    let contract_address = "cc000001"; // 0xcc00000100000000000000000000000000000000
    let contract_balance = U256::from(0xabab12);
    let mut builder = Code::builder();
    let code = builder
        .append_code(&mut get_code_for_call(50_000, contract_address, 0, 0, 0x00, 0, 0x20))
        // call cost = 2621 + 3(=memory expansion)
        // exec cost = 20
        // total = 2644
        .append("60206000") // 6
        .append(OpCode::RETURN)
        .clone();
    
    let gas_limit = 100_000;

    let mut tester = EvmEmulator::new_stateful_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .with_contract_deployed(contract_address, contract, contract_balance)
        .run_code(code);
    
    result.expect_status(StatusCode::Success)
        .expect_output("0000000000000000000000000000000000000000000000000000000000abab12")
        .expect_gas(2650);
}

#[test]
fn test_remote_address() {
    let mut builder = Code::builder();
    let contract = builder
        .append(OpCode::ADDRESS)    // get contract address
        .append("6000")
        .append(OpCode::MSTORE)
        .append("60206000")
        .append(OpCode::RETURN)
        .clone();   // = 17

    let contract_address = "cc000001";
    let contract_balance = U256::from(0xabab12);
    let mut builder = Code::builder();
    let code = builder
        .append_code(&mut get_code_for_call(50_000, contract_address, 0, 0, 0x00, 0, 0x20))
        // call cost = 2621 + 3(=memory expansion)
        // exec cost = 17
        // total = 2641
        .append("60206000") // 6
        .append(OpCode::RETURN)
        .clone();
    
    let gas_limit = 100_000;
    let mut tester = EvmEmulator::new_stateful_with(get_default_context());
    let result = tester.with_to(default_address())
        .with_gas_limit(gas_limit)
        .with_gas_left(gas_limit)
        .with_contract_deployed(contract_address, contract, contract_balance)
        .run_code(code);
    
    result.expect_status(StatusCode::Success)
        .expect_output("000000000000000000000000cc00000100000000000000000000000000000000")
        .expect_gas(2647);
}

fn scope_code(depth: u64, max_depth: u64) -> Code {
    let mut sstore = Code::builder()
        .append(OpCode::PUSH32) // 3
        .append(U256::from(depth))  // value
        .append(OpCode::PUSH1) // 3
        .append(0x00)   // key
        .append(OpCode::SSTORE) // 20000(init to non-zero) + 2100(cold)
        .clone();   // 22106

    let mut scope_code = 
        if depth < max_depth {
            Code::builder()
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
                .append(address(depth))   // address
                .append(OpCode::GAS)
                .append(OpCode::CALL)
                .clone()    // 3 * 6 + 2 + call = 20 + call[=3+2600] = 2623
        } else {
            Code::builder()
                .append(OpCode::PUSH1)  // 3
                .append(0x20)
                .append(OpCode::PUSH1)  // 3
                .append(0x00)
                .append(OpCode::RETURN) // 3 = memory expansion
                .clone() // = 9
        };
    
    let code = Code::builder()
        .append_code(&mut sstore)// 22106
        .append_code(&mut scope_code)

        .append(OpCode::PUSH1)// 3
        .append(0x00)
        .append(OpCode::MLOAD)// 3

        .append(OpCode::PUSH32)// 3
        .append(U256::from(depth))
        .append(OpCode::ADD)// 3

        .append(OpCode::PUSH1) // 3
        .append(0x00)
        .append(OpCode::MSTORE)// 3

        .append(OpCode::PUSH1) // 3
        .append(0x20)
        .append(OpCode::PUSH1) // 3
        .append(0x00)
        .append(OpCode::RETURN)
        .clone(); // 24 + 22106 + 2623 for 0..11
    code
}
fn address(num: u64) -> Address {
    Address::from_low_u64_be(0xeeeeee00 + num)
}

#[test]
fn test_deep() {
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
        .append(address(0))   // address
        .append(OpCode::GAS)    // 2
        .append(OpCode::CALL)
        .append(OpCode::PUSH1)
        .append(0x20)   // ret_size
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_offset
        .append(OpCode::RETURN)
        .clone();   // 3 * 6 + call + 3 * 2 + 2 =  24 + call[=3+2600] + 2 = 2629
    
    let mut tester = EvmEmulator::new_stateful_with(get_default_context());
    tester.with_default_gas();

    let max_depth = 50;

    for i in 0..max_depth {
        tester.with_contract_deployed2(address(i), scope_code(i+1, max_depth), U256::zero());
    }
    let result = tester.run_code(code);
    
    result.expect_status(StatusCode::Success);
    for i in 0..max_depth {
        result.expect_storage(address(i), U256::zero(), U256::from(i+1));
    }
    //result.expect_output("0000000000000000000000000000000000000000000000000000000000000042"); // 0x42 = 66 = 1 + 2 + ... + 11)
    result.expect_gas_refund(0);

    // 2629
    //  + 11 * (24 + 22106 + 2623) = 272283
    //  + 22106 + 9
    result.expect_gas(2629 + (max_depth as i64 - 1) * (24 + 22106 + 2623) + 22106 + 9);
}

#[test]
fn test_transfer_to_existent() {
    let sender_address = address(0xff);
    let sender_balance = U256::from_str("ffffffffffffffff").unwrap();
    let receiver_address = address(0xdd);
    let value = U256::from_str("ffffffffffffffff").unwrap();

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
        .append(value)   // value
        .append(OpCode::PUSH20)
        .append(receiver_address)   // address
        .append(OpCode::GAS)    // 2
        .append(OpCode::CALL)
        .append(OpCode::PUSH1)
        .append(0x20)   // ret_size
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_offset
        .append(OpCode::RETURN)
        .clone();   // 3 * 6 + call + 3 * 2 + 2 = 24 + call[=3+2600+25000+9000-2300] + 2 = 2629
        /*
        3       : memory expansion
        2600    : cold access
        9000    : non-zero
        -2300   : gas stipend
        */
    
    let mut emulator = EvmEmulator::new_stateful_with(get_default_context());

    emulator
        .with_default_gas()
        .with_to(sender_address)
        .with_account(sender_address, sender_balance)
        .with_contract_deployed2(receiver_address, Code::empty(), U256::zero());
    
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_output("0000000000000000000000000000000000000000000000000000000000000000")
        .expect_balance(sender_address, sender_balance - value)
        .expect_balance(receiver_address, value)
        .expect_gas(24 + 3+2600+9000-2300 + 2)
        ;
}

#[test]
fn test_transfer_to_non_existent() {
    let sender_address = address(0xff);
    let sender_balance = U256::from_str("ffffffffffffffff").unwrap();
    let receiver_address = address(0xdd);
    let value = U256::from_str("ffffffffffffffff").unwrap();

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
        .append(value)   // value
        .append(OpCode::PUSH20)
        .append(receiver_address)   // address
        .append(OpCode::GAS)    // 2
        .append(OpCode::CALL)
        .append(OpCode::PUSH1)
        .append(0x20)   // ret_size
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_offset
        .append(OpCode::RETURN)
        .clone();   // 3 * 6 + call + 3 * 2 + 2 = 24 + call[=3+2600+25000+9000-2300] + 2 = 2629
        /*
        3       : memory expansion
        2600    : cold access
        25000   : to empty account
        9000    : non-zero
        -2300   : gas stipend
        */
    
    let mut emulator = EvmEmulator::new_stateful_with(get_default_context());

    emulator
        .with_default_gas()
        .with_to(sender_address)
        .with_account(sender_address, sender_balance);
    
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_output("0000000000000000000000000000000000000000000000000000000000000000")
        .expect_balance(sender_address, sender_balance - value)
        .expect_balance(receiver_address, value)
        .expect_gas(24 + 3+2600+25000+9000-2300 + 2)
        ;
}

fn call_value(to: Address, value: U256) -> Code {
    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::PUSH32)
        .append(value)
        .append(OpCode::PUSH20)
        .append(to)
        .append(OpCode::GAS)
        .append(OpCode::CALL)
        .append(OpCode::PUSH1)
        .append(0x20)   // ret_size
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_offset
        .append(OpCode::RETURN)
        .clone();
    code
}

#[test]
fn test_transfer_to_transfer() {
    let a_address = address(0xaa);
    let a_balance = U256::from_str("ffffffffffffffff").unwrap();
    let b_address = address(0xbb);
    let c_address = address(0xcc);
    let value = U256::from_str("ffffffffffffffff").unwrap();

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
        .append(value)   // value
        .append(OpCode::PUSH20)
        .append(b_address)   // address
        .append(OpCode::GAS)    // 2
        .append(OpCode::CALL)
        .append(OpCode::PUSH1)
        .append(0x20)   // ret_size
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_offset
        .append(OpCode::RETURN)
        .clone();
    
    let mut emulator = EvmEmulator::new_stateful_with(get_default_context());

    emulator
        .with_default_gas()
        .with_to(a_address)
        .with_account(a_address, a_balance)
        .with_contract_deployed2(b_address, call_value(c_address, value - 1), U256::zero());
    
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Success)
        .expect_output("0000000000000000000000000000000000000000000000000000000000000000")
        .expect_balance(a_address, a_balance - value)
        .expect_balance(b_address, U256::one())
        .expect_balance(c_address, value - 1)
        // .expect_gas(24 + 3+2600+25000+9000-2300 + 2)
        ;
}

#[test]
fn test_transfer_insufficient_balance() {
    let sender_address = address(0xff);
    let sender_balance = U256::from_str("ffffffffffffffff").unwrap();
    let receiver_address = address(0xdd);
    let value = U256::from_str("ffffffffffffffff").unwrap() + 1;    // balance + 1

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
        .append(value)   // value
        .append(OpCode::PUSH20)
        .append(receiver_address)   // address
        .append(OpCode::GAS)    // 2
        .append(OpCode::CALL)
        .append(OpCode::PUSH1)
        .append(0x20)   // ret_size
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_offset
        .append(OpCode::RETURN)
        .clone();   // 3 * 6 + call + 3 * 2 + 2 = 24 + call[=3+2600+25000+9000-2300] + 2 = 2629
        /*
        3       : memory expansion
        2600    : cold access
        9000    : non-zero
        -2300   : gas stipend
        */
    
    let mut emulator = EvmEmulator::new_stateful_with(get_default_context());

    emulator
        .with_default_gas()
        .with_to(sender_address)
        .with_account(sender_address, sender_balance)
        .with_contract_deployed2(receiver_address, Code::empty(), U256::zero());
    
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Failure(FailureKind::InsufficientBalance))
        .expect_output("")
        ;
}

#[test]
fn test_transfer_gas_by_revisions() {
    let sender_address = address(0xff);
    let sender_balance = U256::from_str("ffffffffffffffff").unwrap();
    let receiver_address = address(0xdd);
    let value = U256::from_str("ffffffffffffffff").unwrap();

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
        .append(value)   // value
        .append(OpCode::PUSH20)
        .append(receiver_address)   // address
        .append(OpCode::PUSH32)  // 3
        .append(U256::from_str("ffff").unwrap())   // gas
        .append(OpCode::CALL)
        .append(OpCode::PUSH1)
        .append(0x20)   // ret_size
        .append(OpCode::PUSH1)
        .append(0x00)   // ret_offset
        .append(OpCode::RETURN)
        .clone();   // 3 * 7 + call + 3 * 2 + 2 = 27 + call[=3+static_cost+cold_cost+9000-2300]

    for revision in Revision::iter() {
        for is_cold in [true, false] {
            let mut emulator = EvmEmulator::new_stateful_with(get_default_context());

            let emulator = emulator
                .with_default_gas()
                .with_to(sender_address)
                .with_account(sender_address, sender_balance)
                .with_contract_deployed2(receiver_address, Code::empty(), U256::zero());
            
            if !is_cold {
                emulator.with_warm_account(receiver_address);
            }

            let result = emulator.run_code_as(code.clone(), revision);
        
            let static_cost = 
                if revision >= Revision::Berlin {
                    0
                }else{
                    if revision <= Revision::Homestead {
                        40
                    }else{
                        700
                    }
                };

            let cold_cost =
                if revision >= Revision::Berlin {
                    if is_cold { 2600 } else { 100 }
                } else {
                    0
                };
    
            result.expect_status(StatusCode::Success)
                .expect_output("0000000000000000000000000000000000000000000000000000000000000000")
                .expect_balance(sender_address, sender_balance - value)
                .expect_balance(receiver_address, value)
                .expect_gas(27 + 3+static_cost+cold_cost+9000-2300)
                ;
        }
    }
}

#[test]
fn test_exceed_call_depth() {
    let code = Code::builder()
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::PUSH32)
        .append(U256::zero())
        .append(OpCode::PUSH20)
        .append(address(0xdd))
        .append(OpCode::GAS)
        .append(OpCode::CALL)
        .append(OpCode::PUSH1)
        .append(0x20)
        .append(OpCode::PUSH1)
        .append(0x00)
        .append(OpCode::RETURN)
        .clone();
    
    let mut emulator = EvmEmulator::new_stateful_with(get_default_context());

    emulator
        .with_default_gas()
        .mutate_scope(|s| {
            s.depth = 1024;
        });
    
    let result = emulator.run_code(code);

    result.expect_status(StatusCode::Failure(FailureKind::CallDepthExceeded))
        .expect_output("");

}