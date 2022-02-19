use bytes::Bytes;

use evmstar::host::host::TransientHost;
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
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PC)
        .append_opcode(OpCode::POP)
        .append_opcode(OpCode::PUSH3)
        .append("000000")
        .append_opcode(OpCode::POP)
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::POP)
        .append_opcode(OpCode::PC)
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::MSTORE)
        .append_opcode(OpCode::PUSH1)
        .append("20")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("000000000000000000000000000000000000000000000000000000000000000a").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(31), output.gas_left);
}

#[test]
pub fn test_jump() {
    let host = TransientHost::new();
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)   // 0
        .append("aa")                // 1
        .append_opcode(OpCode::PUSH1)   // 2
        .append("00")                // 3
        .append_opcode(OpCode::MSTORE)  // 4
        .append_opcode(OpCode::PUSH1)   // 5
        .append(13)                  // 6 (jump to 13)
        .append_opcode(OpCode::JUMP)    // 7
        .append_opcode(OpCode::PUSH1)   // 8
        .append("ff")                // 9
        .append_opcode(OpCode::PUSH1)   // 10
        .append("00")                // 11
        .append_opcode(OpCode::MSTORE)  // 12
        .append_opcode(OpCode::JUMPDEST)// 13
        .append_opcode(OpCode::PUSH1)
        .append("20")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000aa").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(30), output.gas_left);
}

#[test]
pub fn test_jump_bad() {
    let host = TransientHost::new();
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)   // 0
        .append("aa")                // 1
        .append_opcode(OpCode::PUSH1)   // 2
        .append("00")                // 3
        .append_opcode(OpCode::MSTORE)  // 4
        .append_opcode(OpCode::PUSH1)   // 5
        .append(13)                  // 6 (jump to 13)
        .append_opcode(OpCode::JUMP)    // 7
        .append_opcode(OpCode::PUSH1)   // 8
        .append("ff")                // 9
        .append_opcode(OpCode::PUSH1)   // 10
        .append("00")                // 11
        .append_opcode(OpCode::MSTORE)  // 12
        .append_opcode(OpCode::PC)      // 13 (not JUMPDEST)
        .append_opcode(OpCode::PUSH1)
        .append("20")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);

    assert_eq!(StatusCode::Failure(FailureKind::BadJumpDestination), output.status_code);
}


#[test]
pub fn test_jumpi() {
    let host = TransientHost::new();
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)   // 0
        .append("aa")                // 1
        .append_opcode(OpCode::PUSH1)   // 2
        .append("00")                // 3
        .append_opcode(OpCode::MSTORE)  // 4
        .append_opcode(OpCode::PUSH1)   // 5
        .append("01")                // 6 (not zero)
        .append_opcode(OpCode::PUSH1)   // 7
        .append(15)                  // 8 (jumpi to 15)
        .append_opcode(OpCode::JUMPI)   // 9
        .append_opcode(OpCode::PUSH1)   // 10
        .append("ff")                // 11
        .append_opcode(OpCode::PUSH1)   // 12
        .append("00")                // 13
        .append_opcode(OpCode::MSTORE)  // 14
        .append_opcode(OpCode::JUMPDEST)// 15
        .append_opcode(OpCode::PUSH1)
        .append("20")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000aa").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(35), output.gas_left);
}

#[test]
pub fn test_jumpi_condition_unmet() {
    let host = TransientHost::new();
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)   // 0
        .append("aa")                // 1
        .append_opcode(OpCode::PUSH1)   // 2
        .append("00")                // 3
        .append_opcode(OpCode::MSTORE)  // 4
        .append_opcode(OpCode::PUSH1)   // 5
        .append("00")                // 6 (zero)
        .append_opcode(OpCode::PUSH1)   // 7
        .append(15)                  // 8 (jumpi to 15)
        .append_opcode(OpCode::JUMPI)   // 9
        .append_opcode(OpCode::PUSH1)   // 10
        .append("ff")                // 11
        .append_opcode(OpCode::PUSH1)   // 12
        .append("00")                // 13
        .append_opcode(OpCode::MSTORE)  // 14
        .append_opcode(OpCode::JUMPDEST)// 15
        .append_opcode(OpCode::PUSH1)
        .append("20")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    let data = decode("00000000000000000000000000000000000000000000000000000000000000ff").unwrap();

    assert_eq!(StatusCode::Success, output.status_code);
    assert_eq!(Bytes::from(data), output.data);
    assert_eq!(consumed_gas(44), output.gas_left);
}

#[test]
pub fn test_jumpi_bad() {
    let host = TransientHost::new();
    let mut executor = Executor::new_with_tracing(Box::new(host));
    let mut builder = Code::builder();

    let code = builder
        .append_opcode(OpCode::PUSH1)   // 0
        .append("aa")                // 1
        .append_opcode(OpCode::PUSH1)   // 2
        .append("00")                // 3
        .append_opcode(OpCode::MSTORE)  // 4
        .append_opcode(OpCode::PUSH1)   // 5
        .append("01")                // 6 (not zero)
        .append_opcode(OpCode::PUSH1)   // 7
        .append(15)                  // 8 (jumpi to 15)
        .append_opcode(OpCode::JUMPI)   // 9
        .append_opcode(OpCode::PUSH1)   // 10
        .append("ff")                // 11
        .append_opcode(OpCode::PUSH1)   // 12
        .append("00")                // 13
        .append_opcode(OpCode::MSTORE)  // 14
        .append_opcode(OpCode::PC)      // 15 (not JUMPDEST)
        .append_opcode(OpCode::PUSH1)
        .append("20")
        .append_opcode(OpCode::PUSH1)
        .append("00")
        .append_opcode(OpCode::RETURN);
    
    let output = executor.execute_raw(&code);
    assert_eq!(StatusCode::Failure(FailureKind::BadJumpDestination), output.status_code);
}