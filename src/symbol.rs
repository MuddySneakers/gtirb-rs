use anyhow::Result;

use crate::*;

#[derive(Debug)]
pub enum Payload {
    Value(Addr),
    Referent(Uuid),
}

#[derive(Debug, Default)]
pub struct SymbolData {
    pub(crate) parent: Option<Index>,

    uuid: Uuid,
    name: String,
    payload: Option<Payload>,
}

impl SymbolData {
    pub fn new(name: &str) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: name.to_owned(),
            ..Default::default()
        }
    }

    pub(crate) fn load_protobuf(
        context: Rc<RefCell<Context>>,
        message: proto::Symbol,
    ) -> Result<Index> {
        use crate::proto::symbol::OptionalPayload;

        let payload = match message.optional_payload {
            Some(OptionalPayload::Value(n)) => Some(Payload::Value(Addr(n))),
            Some(OptionalPayload::ReferentUuid(bytes)) => {
                Some(Payload::Referent(crate::util::parse_uuid(&bytes)?))
            }
            None => None,
        };

        let symbol = Self {
            parent: None,

            uuid: crate::util::parse_uuid(&message.uuid)?,
            name: message.name,
            payload: payload,
        };

        Ok(context.borrow_mut().symbol.insert(symbol))
    }
}

impl NodeData<Symbol> for SymbolData {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

#[derive(Clone, Debug)]
pub struct Symbol {
    index: Index,
    context: Rc<RefCell<Context>>,
}

impl Symbol {}

impl Node<Symbol, SymbolData> for Symbol {
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

    fn arena(&self) -> Ref<Arena<SymbolData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.symbol)
    }

    fn arena_mut(&self) -> RefMut<Arena<SymbolData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.symbol)
    }
}

impl Child<Module, ModuleData> for Symbol {
    fn parent(&self) -> (Option<Index>, PhantomData<Module>) {
        (self.borrow().parent, PhantomData)
    }

    fn set_parent(&self, (index, _): (Index, PhantomData<Module>)) {
        self.borrow_mut().parent.replace(index);
    }
}
