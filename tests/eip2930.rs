use std::rc::Rc;
use std::cell::RefCell;

use bytes::Bytes;
use ethereum_types::{U256, Address};
use evmstar::model::evmc::AccessList;

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

fn consumed_gas(amount: i64) -> i64 {
    i64::max_value() - amount
}
fn address_0() -> Address {
    Address::from_low_u64_be(0)
}
fn address_1() -> Address {
    Address::from_low_u64_be(1)
}
fn address_2() -> Address {
    Address::from_low_u64_be(2)
}
fn address_default() -> Address {
    Address::from_low_u64_be(0xaaaaaaaa)
}
fn address_ext() -> Address {
    Address::from_low_u64_be(0xffffffff)
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
fn test_eip2930_nocode() {
    {
        let host = StatefulHost::new_with(get_default_context());
        let host = Rc::new(RefCell::new(host));

        let context = CallScope::default();
        let mut executor = Executor::new_with_execution_cost(host.clone(), true, Revision::Berlin);
        
        let mut access_list = AccessList::default();
        access_list.add_account(address_0());
        let output = executor.execute_with_access_list(context, access_list);

        assert_eq!(StatusCode::Success, output.status_code);
        assert_eq!(Bytes::default(), output.data);
        assert_eq!(21000 + 2400, consumed_gas(output.gas_left));
    }
    {
        let host = StatefulHost::new_with(get_default_context());
        let host = Rc::new(RefCell::new(host));

        let context = CallScope::default();
        let mut executor = Executor::new_with_execution_cost(host.clone(), true, Revision::Berlin);
        
        let mut access_list = AccessList::default();
        access_list.add_account(address_0());
        access_list.add_account(address_0());
        access_list.add_account(address_1());
        let output = executor.execute_with_access_list(context, access_list);

        assert_eq!(StatusCode::Success, output.status_code);
        assert_eq!(Bytes::default(), output.data);
        assert_eq!(21000 + 2400*3, consumed_gas(output.gas_left));
    }
}

#[test]
fn test_eip2930_nocode_storage() {
    {
        let host = StatefulHost::new_with(get_default_context());
        let host = Rc::new(RefCell::new(host));
        let mut executor = Executor::new_with_execution_cost(host.clone(), true, Revision::Berlin);
        
        let mut access_list = AccessList::default();
        access_list.add_account(address_0());
        access_list.add_account(address_0());
        access_list.add_account(address_1());
        access_list.add_storage(address_0(), U256::from(0));
    
        let context = CallScope::default();
        let output = executor.execute_with_access_list(context, access_list);
    
        assert_eq!(StatusCode::Success, output.status_code);
        assert_eq!(Bytes::default(), output.data);
        assert_eq!(21000 + 2400*3 + 1900, consumed_gas(output.gas_left));
    }
    {
        let host = StatefulHost::new_with(get_default_context());
        let host = Rc::new(RefCell::new(host));
        let mut executor = Executor::new_with_execution_cost(host.clone(), true, Revision::Berlin);
        
        let mut access_list = AccessList::default();
        access_list.add_account(address_0());
        access_list.add_account(address_0());
        access_list.add_account(address_1());
        access_list.add_storage(address_0(), U256::from(0));
        access_list.add_storage(address_2(), U256::from(0));
    
        let context = CallScope::default();
        let output = executor.execute_with_access_list(context, access_list);
    
        assert_eq!(StatusCode::Success, output.status_code);
        assert_eq!(Bytes::default(), output.data);
        assert_eq!(21000 + 2400*4 + 1900*2, consumed_gas(output.gas_left));
    }
}

#[test]
fn test_eip2930() {
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with_execution_cost(host.clone(), true, Revision::Berlin);
    
    let mut access_list = AccessList::default();
    access_list.add_account(address_default());
    access_list.add_account(address_default());
    access_list.add_account(address_ext());
    access_list.add_storage(address_default(), U256::from(0));

    let mut builder = Code::builder();
    let code = builder
        .append(OpCode::PUSH1)
        .append(0)
        .append(OpCode::SLOAD)  // warm

        .append(OpCode::PUSH1)
        .append(1)
        .append(OpCode::SLOAD)   // cold
        
        .append(OpCode::PUSH4)
        .append("aaaaaaaa")
        .append(OpCode::EXTCODESIZE) // warm

        .append(OpCode::PUSH4)
        .append("ffffffff")
        .append(OpCode::EXTCODESIZE) // warm

        .append(OpCode::PUSH4)
        .append("eeeeeeee")
        .append(OpCode::EXTCODESIZE) // cold
        ;

    let mut context = CallScope::default();
    context.to = address_default();
    context.code = code.clone();

    let output = executor.execute_with_access_list(context, access_list);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);

    let expected = 21000
        + 2400*3    // account access * 3
        + 1900      // storage access * 1
        + 3 * 5     // push * 5
        + 100       // warm sload
        + 2100      // cold sload
        + 100       // warm extcodesize
        + 100       // warm extcodesize
        + 2600      // cold extcodesize
        ;
    assert_eq!(expected, consumed_gas(output.gas_left));
}

#[test]
fn test_eip2930_sstore() {
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(address_default(), U256::from(0), U256::from(1));    // add original value
    
    let mut executor = Executor::new_with_execution_cost(host.clone(), true, Revision::Berlin);
    
    let mut access_list = AccessList::default();
    access_list.add_storage(address_default(), U256::from(0));  // make it warm

    let mut builder = Code::builder();
    let code = builder
        .append(OpCode::PUSH1)
        .append(2)  // new value
        .append(OpCode::PUSH1)
        .append(0)  // warm key
        .append(OpCode::SSTORE)  // warm

        .append(OpCode::PUSH1)
        .append(1)  // new value
        .append(OpCode::PUSH1)
        .append(1)  // cold key
        .append(OpCode::SSTORE)  // cold
        ;
    
    let mut context = CallScope::default();
    context.to = address_default();
    context.code = code.clone();

    let output = executor.execute_with_access_list(context, access_list);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);

    let expected = 21000
        + 1900      // storage access * 1
        + 2400      // account(to) access * 1
        + 3 * 4     // push * 2
        + 2900      // warm sstore
        + 22100     // cold sstore
        ;
    assert_eq!(expected, consumed_gas(output.gas_left));
}