use ethereum_types::Address;
use num::traits::FromPrimitive;

use crate::model::{
    opcode::OpCode,
    code::{
        CodeError
    }
};
use crate::executor::{
    callstack::CallContext,
};
use crate::resume::{
    Resume,
};
use crate::interpreter::{
    stack::{Stack, StackOperationError, Memory},
    Interrupt,
};
use crate::utils::{
    u256_to_address,
};

pub struct Interpreter {
    pub pc: usize,
    pub stack: Stack,

}

impl Interpreter {
    pub fn resume_interpret(&self, resume: Resume, context: &mut CallContext) -> Result<Interrupt, InterpreterError> {
        self.apply_resume(resume, &mut context.stack, &mut context.memory);
        
        loop {
            let byte = context.code.try_get(context.pc).map_err(|e| InterpreterError::CodeError(e))?;
            if let Some(opcode) = OpCode::from_u8(byte) {
                match self.next_instruction(&opcode, &mut context.stack, &mut context.memory)? {
                    None => (),
                    Some(i) => {
                        return Ok(i)
                    }
                };
            }
            
            context.pc += 1;
        }
    }

    /// resume interpretation with returned value.
    fn apply_resume(&self, resume: Resume, stack: &mut Stack, memory: &mut Memory) {
        match resume {
            Resume::Init => (),
            Resume::Balance(balance) => {
                stack.push(balance)
            }
        }
    }

    fn next_instruction(&self, opcode: &OpCode, stack: &mut Stack, memory: &mut Memory) -> Result<Option<Interrupt>, InterpreterError> {
        match opcode {
            OpCode::STOP => Ok(Some(Interrupt::Return)),

            OpCode::BALANCE => {
                let address = stack.pop().map_err(|e| InterpreterError::StackOperationError(e))?;
                let address = u256_to_address(address);
                Ok(Some(Interrupt::Balance(address)))
            }

            _ => Ok(None)
        }
    }
}

pub enum InterpreterError {
    StackOperationError(StackOperationError),
    CodeError(CodeError)
}