use crate::*;

pub struct IR {
    uuid: Uuid,
    version: u32,
    modules: Vec<Option<module::ModuleData>>,
    sections: Vec<Option<section::SectionData>>,
    byte_intervals: Vec<Option<byte_interval::ByteIntervalData>>,
    code_blocks: Vec<Option<code_block::CodeBlockData>>,
    data_blocks: Vec<Option<data_block::DataBlockData>>,
    sym_exprs: Vec<Option<symbolic_expression::SymbolicExpressionData>>,
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
        self.modules.push(Some(module::ModuleData::new(name)));
        module::Module::new(key)
    }

    pub(crate) fn get_module_data(&self, module: &module::Module) -> &module::ModuleData {
        self.modules[module.key].as_ref().expect("Attempt to access deleted Module!")
    }

    pub(crate) fn get_module_data_mut(&mut self, module: &module::Module) -> &mut module::ModuleData {
        self.modules[module.key].as_mut().expect("Attempt to access deleted Module!")
    }

    pub(crate) fn get_section_data(&self, section: &section::Section) -> &section::SectionData {
        self.sections[section.key].as_ref().expect("Attempt to access deleted Section!")
    }

    pub(crate) fn get_section_data_mut(&mut self, section: &section::Section) -> &mut section::SectionData {
        self.sections[section.key].as_mut().expect("Attempt to access deleted Section!")
    }

    pub(crate) fn get_byte_interval_data(&self, bi: &byte_interval::ByteInterval) -> &byte_interval::ByteIntervalData {
        self.byte_intervals[bi.key].as_ref().expect("Attempt to access deleted ByteInterval!")
    }

    pub(crate) fn get_code_block_data(&self, cb: &code_block::CodeBlock) -> &code_block::CodeBlockData {
        self.code_blocks[cb.key].as_ref().expect("Attempt to access deleted CodeBlock!")
    }

    pub fn add_section(&mut self) -> section::Section {
        let key = self.sections.len();
        self.sections.push(Some(section::SectionData::new()));
        section::Section::new(key)
    }
}
