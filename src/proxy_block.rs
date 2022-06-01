use anyhow::Result;

use crate::*;

#[derive(Default, Debug, PartialEq)]
pub struct ProxyBlockData {
    pub(crate) parent: Option<Index>,

    uuid: Uuid,
}

impl ProxyBlockData {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            ..Default::default()
        }
    }
    pub(crate) fn load_protobuf(
        context: Rc<RefCell<Context>>,
        message: proto::ProxyBlock,
    ) -> Result<Index> {
        let proxy_block = Self {
            parent: None,
            uuid: crate::util::parse_uuid(&message.uuid)?,
        };
        Ok(context.borrow_mut().proxy_block.insert(proxy_block))
    }
}

impl NodeData<ProxyBlock> for ProxyBlockData {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

#[derive(Clone, Debug)]
pub struct ProxyBlock {
    index: Index,
    context: Rc<RefCell<Context>>,
}

impl ProxyBlock {}

impl Node<ProxyBlock, ProxyBlockData> for ProxyBlock {
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

    fn arena(&self) -> Ref<Arena<ProxyBlockData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.proxy_block)
    }

    fn arena_mut(&self) -> RefMut<Arena<ProxyBlockData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.proxy_block)
    }
}

impl Child<Module, ModuleData> for ProxyBlock {
    fn parent(&self) -> (Option<Index>, PhantomData<Module>) {
        (self.borrow().parent, PhantomData)
    }

    fn set_parent(&self, (index, _): (Index, PhantomData<Module>)) {
        self.borrow_mut().parent.replace(index);
    }
}
