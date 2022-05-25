use crate::*;

pub(crate) struct CodeBlockData {
    pub(crate) byte_interval: byte_interval::ByteInterval,
}

#[derive(Clone, Copy)]
pub struct CodeBlock {
    pub(crate) key: usize,
}

impl CodeBlock {
    fn get_address(&self, ir: &ir::IR) -> u64 {
        let bi = ir.get_code_block_data(self).byte_interval;
        ir.get_byte_interval_data(&bi).address
    }
}
