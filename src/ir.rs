use std::path::Path;

use anyhow::Result;
use prost::Message;

use crate::*;

#[derive(Debug, Default, PartialEq)]
pub struct IRData {
    uuid: Uuid,
    version: u32,
    modules: Vec<Index>,
}

impl IRData {
    pub fn new() -> IR {
        let mut context = Context::new();
        let index = context.ir.insert(Self {
            uuid: Uuid::new_v4(),
            version: 1,
            modules: Vec::new(),
        });
        IR {
            index,
            context: Rc::new(RefCell::new(context)),
        }
    }

    pub fn read<P: AsRef<Path>>(path: P) -> Result<IR> {
        let bytes = std::fs::read(path)?;
        IRData::load_protobuf(proto::Ir::decode(&*bytes)?)
    }

    fn load_protobuf(message: proto::Ir) -> Result<IR> {
        let context = Rc::new(RefCell::new(Context::new()));

        let ir = Self {
            uuid: crate::util::parse_uuid(&message.uuid)?,
            version: message.version,
            modules: message
                .modules
                .into_iter()
                .map(|m| ModuleData::load_protobuf(context.clone(), m))
                .collect::<Result<Vec<Index>>>()?,
        };

        let index = context.borrow_mut().ir.insert(ir);

        for (_, module) in context.borrow_mut().module.iter_mut() {
            module.parent.replace(index);
        }

        Ok(IR {
            index: index,
            context: context,
        })
    }
}

impl NodeData<IR> for IRData {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

#[derive(Clone, Debug)]
pub struct IR {
    index: Index,
    context: Rc<RefCell<Context>>,
}

impl IR {
    pub fn find_node<U, UData>(&self, uuid: Uuid) -> Option<U>
    where
        U: Node<U, UData>,
        UData: NodeData<U>,
    {
        self.context
            .borrow()
            .uuid_map
            .get(&uuid)
            .map(|index| U::new(*index, self.context.clone()))
            .filter(|node| node.arena().contains(node.index()))
            .filter(|node| node.borrow().uuid() == uuid)
    }

    pub fn version(&self) -> u32 {
        self.borrow().version
    }

    pub fn set_version(&self, version: u32) {
        self.borrow_mut().version = version
    }

    pub fn modules(&self) -> NodeIterator<Module, ModuleData> {
        self.node_iter()
    }

    pub fn add_module(&self, module: ModuleData) -> Module {
        self.add_node(module)
    }

    pub fn remove_module(&self, node: Module) {
        self.remove_node(node);
    }
}

impl Node<IR, IRData> for IR {
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

    fn arena(&self) -> Ref<Arena<IRData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.ir)
    }
    fn arena_mut(&self) -> RefMut<Arena<IRData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.ir)
    }
}

impl Parent<Module, ModuleData> for IR {
    fn nodes(&self) -> Ref<Vec<Index>> {
        Ref::map(self.borrow(), |ir| &ir.modules)
    }

    fn nodes_mut(&self) -> RefMut<Vec<Index>> {
        RefMut::map(self.borrow_mut(), |ir| &mut ir.modules)
    }

    fn child_arena(&self) -> Ref<Arena<ModuleData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.module)
    }

    fn child_arena_mut(&self) -> RefMut<Arena<ModuleData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.module)
    }
}

impl PartialEq for IR {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Rc::ptr_eq(&self.context, &other.context)
    }
}

pub fn read<P: AsRef<Path>>(path: P) -> Result<IR> {
    let bytes = std::fs::read(path)?;
    IRData::load_protobuf(proto::Ir::decode(&*bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_ir() {
        let ir = IRData::new();
        assert_eq!(ir.version(), 1);
        assert_eq!(ir.modules().count(), 0);
    }

    #[test]
    fn new_ir_is_unique() {
        assert_ne!(IRData::new(), IRData::new());
    }

    #[test]
    fn can_set_version() {
        let ir = IRData::new();
        ir.set_version(42);
        assert_eq!(ir.version(), 42);
    }

    #[test]
    fn can_add_new_module() {
        let ir = IRData::new();
        let module = ModuleData::new("dummy");
        ir.add_module(module);
        let module = ir.modules().nth(0);
        assert!(module.is_some());
        assert_eq!(module.unwrap().ir(), ir);
    }

    #[test]
    fn can_remove_module() {
        let ir = IRData::new();
        let module = ModuleData::new("dummy");
        let module = ir.add_module(module);
        let uuid = module.uuid();

        ir.remove_module(module);
        assert_eq!(ir.modules().count(), 0);

        let node: Option<Module> = ir.find_node(uuid);
        assert!(node.is_none());
    }

    #[test]
    fn can_find_node_by_uuid() {
        let ir = IRData::new();
        let module = ModuleData::new("dummy");
        let uuid = module.uuid();
        ir.add_module(module);
        let node: Option<Module> = ir.find_node(uuid);
        assert!(node.is_some());
        assert_eq!(uuid, node.unwrap().uuid());
    }

    #[test]
    fn can_modify_modules() {
        let ir = IRData::new();
        ir.add_module(ModuleData::new("foo"));
        ir.add_module(ModuleData::new("bar"));
        for module in ir.modules() {
            module.set_preferred_address(Addr(1));
        }
        assert!(ir.modules().all(|m| m.preferred_address() == 1.into()));
    }
}
