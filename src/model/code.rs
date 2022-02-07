#[derive(Clone, Debug)]
pub struct Code(pub Vec<u8>);

impl Code {
    pub fn try_get(&self, pc: usize) -> Result<u8, CodeError> {
        if let Some(byte) = self.0.get(pc) {
            return Ok(*byte);
        }
        Err(CodeError::Overflow)
    }

    pub fn slice(&self, offset: usize, size: usize) -> &[u8] {
        &self.0[offset..(offset+size)]
    }
}

pub enum CodeError {
    Overflow,
}