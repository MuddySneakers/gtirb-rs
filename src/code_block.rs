use crate::*;

#[derive(Default, Debug, PartialEq)]
pub struct CodeBlockData {
    pub(crate) parent: Option<Index>,

    uuid: Uuid,
}

impl CodeBlockData {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            ..Default::default()
        }
    }
}

impl NodeData<CodeBlock> for CodeBlockData {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

#[derive(Clone, Debug)]
pub struct CodeBlock {
    index: Index,
    context: Rc<RefCell<Context>>,
}

impl CodeBlock {}

impl Node<CodeBlock, CodeBlockData> for CodeBlock {
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

    fn arena(&self) -> Ref<Arena<CodeBlockData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.code_block)
    }

    fn arena_mut(&self) -> RefMut<Arena<CodeBlockData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.code_block)
    }
}

impl Child<ByteInterval, ByteIntervalData> for CodeBlock {
    fn parent(&self) -> (Option<Index>, PhantomData<ByteInterval>) {
        (self.borrow().parent, PhantomData)
    }

    fn set_parent(&self, (index, _): (Index, PhantomData<ByteInterval>)) {
        self.borrow_mut().parent.replace(index);
    }
}
