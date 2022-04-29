use crate::*;

pub struct IR {
    uuid: Uuid,
    version: u32,
    pub(crate) modules: Vec<module::ModuleData>,
    pub(crate) sections: Vec<section::SectionData>,
    pub(crate) byte_intervals: Vec<byte_interval::ByteIntervalData>,
    pub(crate) code_blocks: Vec<code_block::CodeBlockData>,
    pub(crate) data_blocks: Vec<data_block::DataBlockData>,
    pub(crate) sym_exprs: Vec<symbolic_expression::SymbolicExpressionData>,
}

impl IR {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            version: 1,
            modules: Vec::new(),
            sections: Vec::new(),
            byte_intervals: Vec::new(),
            code_blocks: Vec::new(),
            data_blocks: Vec::new(),
            sym_exprs: Vec::new(),
        }
    }

    pub fn add_module(&mut self, name: &str) -> module::Module {
        let key = self.modules.len();
        self.modules.push(module::ModuleData::new(name));
        module::Module::new(key)
    }
}
