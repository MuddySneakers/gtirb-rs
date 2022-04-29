use crate::*;

pub(crate) struct ModuleData {
    uuid: Uuid,
    name: String,
    // ...

    sections: Vec<section::Section>,
}

impl ModuleData {
    // Note that new() is only invoked from within the crate, not by clients!
    pub(crate) fn new(name: &str) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: name.to_string(),
            sections: Vec::new(),
        }
    }
}

pub struct Module {
    key: usize,
}

impl Module {
    // Note that new() is only invoked from within the crate, not by clients!
    pub(crate) fn new(key: usize) -> Self {
        Self {
            key: key,
        }
    }

    pub fn get_uuid(&self, ir: &ir::IR) -> Uuid {
        ir.modules[self.key].uuid
    }

    pub fn get_name<'a>(&self, ir: &'a ir::IR) -> &'a str {
        &ir.modules[self.key].name[..]
    }

    pub fn add_section(&self, ir: &mut ir::IR) -> section::Section {
        let key = ir.sections.len();
        ir.sections.push(section::SectionData::new());
        section::Section::new(key)
    }
}
