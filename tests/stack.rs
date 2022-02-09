use bytes::Bytes;

use evmstar::executor::callstack::{CallStack, CallContext};
use evmstar::interpreter::{
    Interrupt,
    interpreter::Interpreter
};
use evmstar::host::host::TransientHost;
use evmstar::interpreter::stack::{Stack};
use evmstar::executor::executor::Executor;
use evmstar::model::code::{Code};
use evmstar::model::opcode::OpCode;



#[test]
pub fn test_stack() {
    let stack = Stack::default();

    // let interpreter = Interpreter::new();
    // let call_context = CallContext::default();
    let callstack = CallStack::default();

    // let host = TransientHost::new();
    // println!("------ hello world");
    // let host = Box::new(host);
    // let mut executor = Executor::new(host);
    // let mut builder = Code::builder();

    // let code = builder
    //     .append_opcode(OpCode::PUSH1)
    //     .append(&[0xff])
    //     .append_opcode(OpCode::PUSH1)
    //     .append(&[0x00])
    //     .append_opcode(OpCode::MSTORE)
    //     .append_opcode(OpCode::RETURN);
    
    //     println!("code: {:?}", code);

    // let output = executor.execute_raw(&code);

    // assert_eq!(Bytes::from(vec![0xff]), output.data);
}