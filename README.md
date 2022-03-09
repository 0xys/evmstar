# EVM star

# Toy Example
```rs
let host = TransientHost::new();
let mut executor = Executor::new(Box::new(host));
let mut builder = Code::builder();

let code = builder
    .append(OpCode::PUSH1)  // OpCode
    .append("02")           // hex character
    .append(OpCode::PUSH1)
    .append(0x03)           // u8
    .append(OpCode::ADD)
    .append(OpCode::PUSH1)
    .append("00")
    .append(OpCode::MSTORE)
    .append("60206000")     // hex string
    .append(OpCode::RETURN);

let output = executor.execute_raw(&code);

let data = decode("0000000000000000000000000000000000000000000000000000000000000005").unwrap();

assert_eq!(StatusCode::Success, output.status_code);
assert_eq!(Bytes::from(data), output.data);
assert_eq!(consumed_gas(24), output.gas_left);
```

# Progress
- [x] push,pop,dup
- [x] arithmetic opcodes
- [x] memory opcode (mload, mstore, mstore8)
- [x] return
- [x] storage opcode (sload, sstore)
- [x] context opcode
- [ ] call (in progress)
- [ ] revert
- [ ] delegatecall, staticcall, callcode
- [ ] create
- [ ] create2
- [ ] selfdestruct
- [ ] sha3, precompiles