use crate::*;

pub(crate) struct SectionData {
    name: String,
}

impl SectionData {
    pub(crate) fn new() -> Self { Self { name: String::from("") } }
}

#[derive(Clone, Copy)]
pub struct Section {
    pub(crate) key: usize,
}

impl Section {
    pub(crate) fn new(key: usize) -> Self {
        Self {
            key: key,
        }
    }

    pub fn set_name(&mut self, ir: &mut ir::IR, name: String) {
        ir.get_section_data_mut(self).name = name;
    }
}