use crate::*;

#[derive(Default, Debug, PartialEq)]
pub struct DataBlockData {
    pub(crate) parent: Option<Index>,

    uuid: Uuid,
}

impl DataBlockData {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            ..Default::default()
        }
    }
}

impl NodeData<DataBlock> for DataBlockData {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

#[derive(Clone, Debug)]
pub struct DataBlock {
    index: Index,
    context: Rc<RefCell<Context>>,
}

impl DataBlock {}

impl Node<DataBlock, DataBlockData> for DataBlock {
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

    fn arena(&self) -> Ref<Arena<DataBlockData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.data_block)
    }

    fn arena_mut(&self) -> RefMut<Arena<DataBlockData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.data_block)
    }
}

impl Child<ByteInterval, ByteIntervalData> for DataBlock {
    fn parent(&self) -> (Option<Index>, PhantomData<ByteInterval>) {
        (self.borrow().parent, PhantomData)
    }

    fn set_parent(&self, (index, _): (Index, PhantomData<ByteInterval>)) {
        self.borrow_mut().parent.replace(index);
    }
}
