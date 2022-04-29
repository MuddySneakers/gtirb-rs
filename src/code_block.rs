use crate::*;

pub(crate) struct CodeBlockData {
    pub(crate) byte_interval: usize,  // Index in the IR's byte_interval array
}

pub struct CodeBlock {
    key: usize,
}

impl CodeBlock {
    fn get_address(&self, ir: &ir::IR) -> u64 {
        let bi_key = ir.code_blocks[self.key].byte_interval;
        ir.byte_intervals[bi_key].address
    }
}
