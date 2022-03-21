use std::cell::RefCell;
use std::rc::Rc;

use evmstar::executor::callstack::CallScope;
use evmstar::host::host::{
    TransientHost,
};
use evmstar::executor::{
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

fn consumed_gas(amount: i64) -> i64 {
    i64::max_value() - amount
}

#[test]
fn test_empty_with_intrinsic_cost() {
    let mut builder = Code::builder();
    let code = builder.append("");

    for revision in Revision::iter() {
        let host = TransientHost::new();
        let host = Rc::new(RefCell::new(host));
        let mut executor = Executor::new_with_execution_cost(host.clone(), true, revision);

        let output = executor.execute_raw(code);
        assert_eq!(StatusCode::Success, output.status_code);
        assert_eq!(21000, consumed_gas(output.gas_left));
    }
}

#[test]
fn test_calldata_cost() {
    for revision in Revision::iter() {
        let host = TransientHost::new();
        let host = Rc::new(RefCell::new(host));
        let mut executor = Executor::new_with_execution_cost(host.clone(), true, revision);

        let mut context = CallScope::default();
        let mut vec: Vec<u8> = Vec::new();
        for i in 0..0xff {
            vec.push(i);
        }
        vec.push(0xff);
        context.calldata = Calldata{0: vec};

        let output = executor.execute_raw_with(context);
        assert_eq!(StatusCode::Success, output.status_code);

        let expected_cost = 4 + 255 * 
            if revision >= Revision::Istanbul {
                16
            }else{
                68
            };

        assert_eq!(21000 + expected_cost, consumed_gas(output.gas_left));
    }
}

#[test]
fn test_too_large_code() {
    let mut code: Vec<u8> = vec![];
    for _ in 0..0x6000 {
        code.push(0x00);
    }
    code.push(0x00);
    let mut builder = Code::builder();
    let code = builder.append(code.as_slice());

    for revision in Revision::iter() {
        let host = TransientHost::new();
        let host = Rc::new(RefCell::new(host));
        let mut executor = Executor::new_with(host.clone(), false, revision);
        
        let output = executor.execute_raw(&code);
        if revision >= Revision::Spurious {
            assert_eq!(StatusCode::Failure(FailureKind::OutOfGas), output.status_code);
        }else{
            assert_eq!(StatusCode::Success, output.status_code);
        }
    }
}