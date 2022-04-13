use std::rc::Rc;
use std::cell::RefCell;

use bytes::Bytes;

use evmstar::host::transient::TransientHost;
use evmstar::executor::executor::Executor;
#[allow(unused_imports)]
use evmstar::model::{
    code::{
        Code, Append,
    },
    opcode::OpCode,
    evmc::{
        StatusCode, FailureKind,
    },
};

use hex::{decode};

fn consumed_gas(amount: i64) -> i64 {
    i64::max_value() - amount
}

#[test]
pub fn test_pc() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with_tracing(host.clone());
    let mut builder = Code::builder();

    let code = builder
        .append(OpCode::PC)
        .append(OpCode::POP)
        .append(OpCode::PUSH3)
        .append("000000")
        .append(OpCode::POP)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::POP)
        .append(OpCode::PC)
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::MSTORE)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("000000000000000000000000000000000000000000000000000000000000000a").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(31), output.gas_left);
}

#[test]
pub fn test_jump() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with_tracing(host.clone());
    let mut builder = Code::builder();

    let code = builder
        .append(OpCode::PUSH1)   // 0
        .append("aa")                // 1
        .append(OpCode::PUSH1)   // 2
        .append("00")                // 3
        .append(OpCode::MSTORE)  // 4
        .append(OpCode::PUSH1)   // 5
        .append(13)                  // 6 (jump to 13)
        .append(OpCode::JUMP)    // 7
        .append(OpCode::PUSH1)   // 8
        .append("ff")                // 9
        .append(OpCode::PUSH1)   // 10
        .append("00")                // 11
        .append(OpCode::MSTORE)  // 12
        .append(OpCode::JUMPDEST)// 13
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000aa").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(30), output.gas_left);
}

#[test]
pub fn test_jump_bad() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with_tracing(host.clone());
    let mut builder = Code::builder();

    let code = builder
        .append(OpCode::PUSH1)   // 0
        .append("aa")                // 1
        .append(OpCode::PUSH1)   // 2
        .append("00")                // 3
        .append(OpCode::MSTORE)  // 4
        .append(OpCode::PUSH1)   // 5
        .append(13)                  // 6 (jump to 13)
        .append(OpCode::JUMP)    // 7
        .append(OpCode::PUSH1)   // 8
        .append("ff")                // 9
        .append(OpCode::PUSH1)   // 10
        .append("00")                // 11
        .append(OpCode::MSTORE)  // 12
        .append(OpCode::PC)      // 13 (not JUMPDEST)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Failure(FailureKind::BadJumpDestination), output.status_code);
}


#[test]
pub fn test_jumpi() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with_tracing(host.clone());
    let mut builder = Code::builder();

    let code = builder
        .append(OpCode::PUSH1)   // 0
        .append("aa")                // 1
        .append(OpCode::PUSH1)   // 2
        .append("00")                // 3
        .append(OpCode::MSTORE)  // 4
        .append(OpCode::PUSH1)   // 5
        .append("01")                // 6 (not zero)
        .append(OpCode::PUSH1)   // 7
        .append(15)                  // 8 (jumpi to 15)
        .append(OpCode::JUMPI)   // 9
        .append(OpCode::PUSH1)   // 10
        .append("ff")                // 11
        .append(OpCode::PUSH1)   // 12
        .append("00")                // 13
        .append(OpCode::MSTORE)  // 14
        .append(OpCode::JUMPDEST)// 15
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000aa").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(35), output.gas_left);
}

#[test]
pub fn test_jumpi_condition_unmet() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with_tracing(host.clone());
    let mut builder = Code::builder();

    let code = builder
        .append(OpCode::PUSH1)   // 0
        .append("aa")                // 1
        .append(OpCode::PUSH1)   // 2
        .append("00")                // 3
        .append(OpCode::MSTORE)  // 4
        .append(OpCode::PUSH1)   // 5
        .append("00")                // 6 (zero)
        .append(OpCode::PUSH1)   // 7
        .append(15)                  // 8 (jumpi to 15)
        .append(OpCode::JUMPI)   // 9
        .append(OpCode::PUSH1)   // 10
        .append("ff")                // 11
        .append(OpCode::PUSH1)   // 12
        .append("00")                // 13
        .append(OpCode::MSTORE)  // 14
        .append(OpCode::JUMPDEST)// 15
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000ff").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(44), output.gas_left);
}

#[test]
pub fn test_jumpi_bad() {
    let host = TransientHost::new();
    let host = Rc::new(RefCell::new(host));
    let mut executor = Executor::new_with_tracing(host.clone());
    let mut builder = Code::builder();

    let code = builder
        .append(OpCode::PUSH1)   // 0
        .append("aa")                // 1
        .append(OpCode::PUSH1)   // 2
        .append("00")                // 3
        .append(OpCode::MSTORE)  // 4
        .append(OpCode::PUSH1)   // 5
        .append("01")                // 6 (not zero)
        .append(OpCode::PUSH1)   // 7
        .append(15)                  // 8 (jumpi to 15)
        .append(OpCode::JUMPI)   // 9
        .append(OpCode::PUSH1)   // 10
        .append("ff")                // 11
        .append(OpCode::PUSH1)   // 12
        .append("00")                // 13
        .append(OpCode::MSTORE)  // 14
        .append(OpCode::PC)      // 15 (not JUMPDEST)
        .append(OpCode::PUSH1)
        .append("20")
        .append(OpCode::PUSH1)
        .append("00")
        .append(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    assert_eq!(StatusCode::Failure(FailureKind::BadJumpDestination), output.status_code);
}