use std::rc::Rc;
use std::cell::RefCell;

use bytes::Bytes;
use ethereum_types::{U256, Address};

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


#[test]
/// test case from: https://eips.ethereum.org/EIPS/eip-1283
fn test_eip1283_1(){// 1
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let mut builder = Code::builder();

    let code = builder.append("60006000556000600055");
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(412, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip1283_2(){// 2
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let mut builder = Code::builder();

    let code = builder.append("60006000556001600055");
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20212, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip1283_3(){// 3
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let mut builder = Code::builder();

    let code = builder.append("60016000556000600055");
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20212, consumed_gas(output.gas_left));
    assert_eq!(19800, output.gas_refund);
}

#[test]
fn test_eip1283_4(){// 4
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let mut builder = Code::builder();

    let code = builder.append("60016000556002600055");
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20212, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip1283_5(){// 5
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let mut builder = Code::builder();

    let code = builder.append("60016000556001600055");
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20212, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip1283_6(){// 6
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x01));
    
    let mut builder = Code::builder();
    let code = builder.append("60006000556000600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
    assert_eq!(15000, output.gas_refund);
}

#[test]
fn test_eip1283_7(){// 7
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append("60006000556001600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
    assert_eq!(4800, output.gas_refund);
}

#[test]
fn test_eip1283_8(){// 8
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append("60006000556002600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip1283_9(){// 9
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append("60026000556000600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
    assert_eq!(15000, output.gas_refund);
}

#[test]
fn test_eip1283_10(){// 10
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append("60026000556003600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip1283_11(){// 11
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append("60026000556001600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
    assert_eq!(4800, output.gas_refund);
}

#[test]
fn test_eip1283_12(){// 12
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append("60026000556002600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip1283_13(){// 13
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append("60016000556000600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
    assert_eq!(15000, output.gas_refund);
}

#[test]
fn test_eip1283_14(){// 14
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append("60016000556002600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5212, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip1283_15(){// 15
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append("60016000556001600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(412, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip1283_16(){// 16
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x00));

    let mut builder = Code::builder();
    let code = builder.append("600160005560006000556001600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(40218, consumed_gas(output.gas_left));
    assert_eq!(19800, output.gas_refund);
}

#[test]
fn test_eip1283_17(){// 17
    let host = StatefulHost::new_with(get_default_context());
    let host = Rc::new(RefCell::new(host));
    (*host).borrow_mut().debug_set_storage(default_address(), U256::zero(), U256::from(0x01));

    let mut builder = Code::builder();
    let code = builder.append("600060005560016000556000600055");
    let mut context = CallScope::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(host.clone(), true, Revision::Constantinople);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(10218, consumed_gas(output.gas_left));
    assert_eq!(19800, output.gas_refund);
}