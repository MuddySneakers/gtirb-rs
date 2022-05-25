use crate::*;

pub(crate) struct ByteIntervalData {
    pub(crate) address: u64,
}

#[derive(Clone, Copy)]
pub struct ByteInterval {
    pub(crate) key: usize,
}

impl ByteInterval {
    pub(crate) fn new(key: usize) -> Self {
        Self { key: key }
    }
}