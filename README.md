# EVM star

# Toy Example
```rs
let mut emulator = EvmEmulator::new_transient_with(TxContext::default());
let code = Code::builder()
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
    .clone();

let result = emulator.run_code(code);

result.expect_status(StatusCode::Success)
    .expect_gas(24)
    .expect_output("0000000000000000000000000000000000000000000000000000000000000005");
```

# Progress
- [x] push,pop,dup
- [x] arithmetic opcodes
- [x] memory opcode (mload, mstore, mstore8)
- [x] return
- [x] storage opcode (sload, sstore)
- [x] context opcode
- [x] call
- [x] revert
- [ ] delegatecall, staticcall, callcode
- [ ] create
- [ ] create2
- [ ] selfdestruct
- [ ] sha3, precompiles