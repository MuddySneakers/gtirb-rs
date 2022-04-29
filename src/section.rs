use crate::*;

pub(crate) struct SectionData {
}

impl SectionData {
    pub(crate) fn new() -> Self { Self {} }
}

pub struct Section {
    key: usize,
}

impl Section {
    pub(crate) fn new(key: usize) -> Self {
        Self {
            key: key,
        }
    }
}
