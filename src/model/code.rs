use bytes::Bytes;
use ethereum_types::{
    U256, Address,
};
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
    pub fn empty() -> Self {
        Self {
            0: vec![]
        }
    }

    pub fn append_code<'a>(&'a mut self, code: &mut Code) -> &'a mut Self {
        self.0.append(&mut code.0);
        self
    }

    pub fn get_range(&self, offset: usize, size: usize) -> Vec<u8> {
        if offset + size < self.0.len() {
            let data = &self.0[offset..offset+size];
            Vec::from(data)
        }else{
            let word = &self.0[offset..self.0.len()];
            let padding_size = size - word.len();
            let mut word = Vec::from(word);
            for _ in 0..padding_size {
                word.push(0u8);
            }
            word
        }
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
impl From<Bytes> for Code {
    fn from(bytes: Bytes) -> Self {
        Code {
            0: bytes.to_vec()
        }
    }
}

pub trait Append<T>: Sized {
    fn append<'a>(&'a mut self, _: T) -> &'a mut Self;
}
impl Append<&str> for Code {
    fn append<'a>(&'a mut self, hex: &str) -> &'a mut Self {
        let mut u8a = decode(hex).unwrap();
        self.0.append(&mut u8a);
        self
    }
}
impl Append<&[u8]> for Code {
    fn append<'a>(&'a mut self, data: &[u8]) -> &'a mut Self {
        self.0.append(&mut Vec::from(data));
        self
    }
}
impl Append<Vec<u8>> for Code {
    fn append<'a>(&'a mut self, data: Vec<u8>) -> &'a mut Self {
        let mut data = data.clone();
        self.0.append(&mut data);
        self
    }
}
impl Append<u8> for Code {
    fn append<'a>(&'a mut self, data: u8) -> &'a mut Self {
        self.0.append(&mut Vec::from([data]));
        self
    }
}
impl Append<OpCode> for Code {
    fn append<'a>(&'a mut self, opcode: OpCode) -> &'a mut Self {
        self.0.push(opcode.to_u8());
        self
    }
}
impl Append<U256> for Code {
    fn append<'a>(&'a mut self, value: U256) -> &'a mut Self {
        let mut dst = [0u8; 32];
        value.to_big_endian(&mut dst);
        let data = Vec::from(dst);
        self.append(data);
        self
    }
}
impl Append<Address> for Code {
    fn append<'a>(&'a mut self, address: Address) -> &'a mut Self {
        let data = Vec::from(address.0);
        self.append(data);
        self
    }
}

#[derive(Clone, Debug)]
pub enum CodeError {
    Overflow,
}