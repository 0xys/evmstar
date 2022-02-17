use hex::decode;
use super::opcode::OpCode;

#[derive(Clone, Debug, Default)]
pub struct Code(pub Vec<u8>);

impl Code {
    pub fn slice(&self, offset: usize, size: usize) -> &[u8] {
        &self.0[offset..(offset+size)]
    }

    pub fn builder() -> Self {
        Self {
            0: vec![]
        }
    }

    pub fn append_opcode<'a>(&'a mut self, opcode: OpCode) -> &'a mut Self {
        self.0.push(opcode.to_u8());
        self
    }

    pub fn append<'a>(&'a mut self, data: &[u8]) -> &'a mut Self {
        self.0.append(&mut Vec::from(data));
        self
    }

    pub fn append_code<'a>(&'a mut self, code: &mut Code) -> &'a mut Self {
        self.0.append(&mut code.0);
        self
    }
}

impl From<&str> for Code {
    fn from(hex: &str) -> Self {
        let u8a = decode(hex).unwrap();
        Code {
            0: u8a
        }
    }
}
impl From<&[u8]> for Code {
    fn from(u8a: &[u8]) -> Self {
        Code {
            0: Vec::from(u8a)
        }
    }
}

#[derive(Clone, Debug)]
pub enum CodeError {
    Overflow,
}