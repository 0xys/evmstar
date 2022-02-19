use bytes::Bytes;
use ethereum_types::{U256, Address};

use evmstar::host::stateful::{
    StatefulHost,
};
use evmstar::executor::{
    callstack::CallContext,
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

/// https://eips.ethereum.org/EIPS/eip-2200
#[test]
fn test_eip2200_1(){
    let original = 0x00;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60006000556000600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(1612, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip2200_2(){
    let original = 0x00;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60006000556001600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20812, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip2200_3(){
    let original = 0x00;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60016000556000600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20812, consumed_gas(output.gas_left));
    assert_eq!(19200, output.gas_refund);
}

#[test]
fn test_eip2200_4(){
    let original = 0x00;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60016000556002600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20812, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip2200_5(){
    let original = 0x00;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60016000556001600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(20812, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip2200_6(){
    let original = 0x01;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60006000556000600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5812, consumed_gas(output.gas_left));
    assert_eq!(15000, output.gas_refund);
}

#[test]
fn test_eip2200_7(){
    let original = 0x01;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60006000556001600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5812, consumed_gas(output.gas_left));
    assert_eq!(4200, output.gas_refund);
}

#[test]
fn test_eip2200_8(){
    let original = 0x01;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60006000556002600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5812, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip2200_9(){
    let original = 0x01;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60026000556000600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5812, consumed_gas(output.gas_left));
    assert_eq!(15000, output.gas_refund);
}

#[test]
fn test_eip2200_10(){
    let original = 0x01;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60026000556003600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5812, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip2200_11(){
    let original = 0x01;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60026000556001600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5812, consumed_gas(output.gas_left));
    assert_eq!(4200, output.gas_refund);
}

#[test]
fn test_eip2200_12(){
    let original = 0x01;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60026000556002600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5812, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip2200_13(){
    let original = 0x01;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60016000556000600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5812, consumed_gas(output.gas_left));
    assert_eq!(15000, output.gas_refund);
}

#[test]
fn test_eip2200_14(){
    let original = 0x01;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60016000556002600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(5812, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip2200_15(){
    let original = 0x01;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("60016000556001600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(1612, consumed_gas(output.gas_left));
    assert_eq!(0, output.gas_refund);
}

#[test]
fn test_eip2200_16(){
    let original = 0x00;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("600160005560006000556001600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(40818, consumed_gas(output.gas_left));
    assert_eq!(19200, output.gas_refund);
}

#[test]
fn test_eip2200_17(){
    let original = 0x01;
    let mut host = StatefulHost::new_with(get_default_context());
    host.debug_set_storage(default_address(), U256::zero(), U256::from(original));

    let mut builder = Code::builder();
    let code = builder.append("600060005560016000556000600055");
    let mut context = CallContext::default();
    context.code = code.clone();
    context.to = default_address();

    let mut executor = Executor::new_with(Box::new(host), true, Revision::Istanbul);
    let output = executor.execute_raw_with(context);

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::default(), output.data);
    assert_eq!(10818, consumed_gas(output.gas_left));
    assert_eq!(19200, output.gas_refund);
}