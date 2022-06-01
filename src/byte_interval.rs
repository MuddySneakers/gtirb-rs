use anyhow::Result;

use std::collections::HashMap;

use crate::*;

#[derive(Default, Debug, PartialEq)]
pub struct ByteIntervalData {
    pub(crate) parent: Option<Index>,

    uuid: Uuid,
    size: u64,
    address: Option<Addr>,
    bytes: Vec<u8>,
    code_blocks: Vec<Index>,
    data_blocks: Vec<Index>,
    symbolic_expressions: HashMap<u64, SymbolicExpression>,
}

impl ByteIntervalData {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            ..Default::default()
        }
    }

    pub(crate) fn load_protobuf(
        context: Rc<RefCell<Context>>,
        message: proto::ByteInterval,
    ) -> Result<Index> {
        // let code_blocks = message
        //     .code_blocks
        //     .into_iter()
        //     .map(|m| CodeBlockData::load_protobuf(context.clone(), m))
        //     .collect::<Result<Vec<Index>>>()?;

        // let data_blocks = message
        //     .data_blocks
        //     .into_iter()
        //     .map(|m| DataBlockData::load_protobuf(context.clone(), m))
        //     .collect::<Result<Vec<Index>>>()?;

        let byte_interval = Self {
            parent: None,

            uuid: crate::util::parse_uuid(&message.uuid)?,
            size: message.size,
            address: message.has_address.then(|| Addr(message.address)),
            bytes: message.contents,
            code_blocks: Vec::new(),              // TODO
            data_blocks: Vec::new(),              // TODO
            symbolic_expressions: HashMap::new(), // TODO
        };

        Ok(context.borrow_mut().byte_interval.insert(byte_interval))
    }
}

impl NodeData<ByteInterval> for ByteIntervalData {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

#[derive(Clone, Debug)]
pub struct ByteInterval {
    index: Index,
    context: Rc<RefCell<Context>>,
}

impl ByteInterval {
    pub fn size(&self) -> u64 {
        return self.borrow().size;
    }

    pub fn set_size(&self, n: u64) {
        self.borrow_mut().size = n;
    }

    pub fn address(&self) -> Option<Addr> {
        return self.borrow().address;
    }

    pub fn set_address(&self, address: Option<Addr>) {
        self.borrow_mut().address = address;
    }

    pub fn initialized_size(&self) -> u64 {
        self.borrow().bytes.len() as u64
    }

    pub fn set_initialized_size(&self, n: u64) {
        self.borrow_mut().bytes.resize(n as usize, 0);
        if n > self.size() {
            self.set_size(n);
        }
    }

    pub fn bytes(&self) -> Ref<[u8]> {
        Ref::map(self.borrow(), |i: &ByteIntervalData| &i.bytes[..])
    }

    pub fn set_bytes<T: AsRef<[u8]>>(&self, bytes: T) {
        self.borrow_mut().bytes = bytes.as_ref().to_vec();
    }

    pub fn code_blocks(&self) -> NodeIterator<CodeBlock, CodeBlockData> {
        self.node_iter()
    }

    pub fn add_code_block(&self, code_block: CodeBlockData) -> CodeBlock {
        self.add_node(code_block)
    }

    pub fn remove_code_block(&self, node: CodeBlock) {
        self.remove_node(node);
    }

    pub fn data_blocks(&self) -> NodeIterator<DataBlock, DataBlockData> {
        self.node_iter()
    }

    pub fn add_data_block(&self, data_block: DataBlockData) -> DataBlock {
        self.add_node(data_block)
    }

    pub fn remove_data_block(&self, node: DataBlock) {
        self.remove_node(node);
    }
}

impl Node<ByteInterval, ByteIntervalData> for ByteInterval {
    fn new(index: Index, context: Rc<RefCell<Context>>) -> Self {
        Self { index, context }
    }

    fn index(&self) -> Index {
        self.index
    }

    fn context(&self) -> Rc<RefCell<Context>> {
        self.context.clone()
    }

    fn uuid(&self) -> Uuid {
        self.borrow().uuid
    }

    fn set_uuid(&mut self, uuid: Uuid) {
        self.borrow_mut().uuid = uuid;
    }

    fn arena(&self) -> Ref<Arena<ByteIntervalData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.byte_interval)
    }

    fn arena_mut(&self) -> RefMut<Arena<ByteIntervalData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.byte_interval)
    }
}

impl Child<Section, SectionData> for ByteInterval {
    fn parent(&self) -> (Option<Index>, PhantomData<Section>) {
        (self.borrow().parent, PhantomData)
    }

    fn set_parent(&self, (index, _): (Index, PhantomData<Section>)) {
        self.borrow_mut().parent.replace(index);
    }
}

impl Parent<CodeBlock, CodeBlockData> for ByteInterval {
    fn nodes(&self) -> Ref<Vec<Index>> {
        Ref::map(self.borrow(), |interval| &interval.code_blocks)
    }

    fn nodes_mut(&self) -> RefMut<Vec<Index>> {
        RefMut::map(self.borrow_mut(), |interval| &mut interval.code_blocks)
    }

    fn child_arena(&self) -> Ref<Arena<CodeBlockData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.code_block)
    }

    fn child_arena_mut(&self) -> RefMut<Arena<CodeBlockData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.code_block)
    }
}

impl Parent<DataBlock, DataBlockData> for ByteInterval {
    fn nodes(&self) -> Ref<Vec<Index>> {
        Ref::map(self.borrow(), |interval| &interval.data_blocks)
    }

    fn nodes_mut(&self) -> RefMut<Vec<Index>> {
        RefMut::map(self.borrow_mut(), |interval| &mut interval.data_blocks)
    }

    fn child_arena(&self) -> Ref<Arena<DataBlockData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.data_block)
    }

    fn child_arena_mut(&self) -> RefMut<Arena<DataBlockData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.data_block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_set_attributes() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("dummy"));
        let section = module.add_section(SectionData::new(".dummy"));
        let interval = section.add_byte_interval(ByteIntervalData::new());
        interval.set_size(0xDEAD);
        interval.set_address(Some(Addr(0xBEEF)));
        assert_eq!(interval.size(), 0xDEAD);
        assert_eq!(interval.address(), Some(Addr(0xBEEF)));
    }
}
