use std::collections::HashMap;

use crate::*;

#[derive(Debug, PartialEq)]
pub struct IR {
    uuid: Uuid,
    version: u32,
    modules: HashMap<Uuid, *mut Module>,
}

impl IR {
    pub fn new(context: &mut Context) -> Node<IR> {
        let ir = IR {
            uuid: Uuid::new_v4(),
            version: 1,
            modules: HashMap::new(),
        };
        IR::allocate(context, ir)
    }

    pub fn load_protobuf(
        context: &mut Context,
        message: proto::Ir,
    ) -> Result<Node<IR>> {
        // Load IR protobuf message.
        let ir = IR {
            uuid: crate::util::parse_uuid(&message.uuid)?,
            version: message.version,
            modules: HashMap::new(),
        };
        let mut ir: Node<IR> = IR::allocate(context, ir);

        // Load Module protobuf messages.
        for m in message.modules.into_iter() {
            let module = Module::load_protobuf(context, m)?;
            ir.add_module(module);
        }

        Ok(ir)
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn set_version(&mut self, version: u32) {
        self.version = version;
    }
}

impl Unique for IR {
    fn uuid(&self) -> Uuid {
        self.uuid
    }

    fn set_uuid(&mut self, uuid: Uuid) {
        self.uuid = uuid;
    }
}

impl Node<IR> {
    pub fn add_module(&mut self, mut module: Node<Module>) -> Node<Module> {
        module.deref_mut().parent = Some(self.inner);
        self.modules.insert(module.uuid(), module.inner);
        module
    }

    pub fn remove_module(&mut self, uuid: Uuid) -> Option<Node<Module>> {
        if let Some(ptr) = self.deref_mut().modules.remove(&uuid) {
            let mut module = Node::new(&self.context, ptr);
            module.parent = None;
            Some(module)
        } else {
            None
        }
    }

    pub fn modules(&self) -> Iter<Module> {
        Iter {
            iter: self.modules.iter(),
            context: &self.context,
        }
    }
}

impl Index for IR {
    fn insert(context: &mut Context, node: Self) -> *mut Self {
        let uuid = node.uuid();
        let ptr = Box::into_raw(Box::new(node));
        context.index.borrow_mut().ir.insert(uuid, ptr);
        ptr
    }

    fn remove(context: &mut Context, ptr: *mut Self) -> Option<Box<Self>> {
        let mut ir = unsafe { Box::from_raw(ptr) };
        context.index.borrow_mut().ir.remove(&ir.uuid);
        for ptr in ir.modules.values_mut() {
            Module::remove(context, *ptr);
        }
        Some(ir)
    }

    fn search(context: &Context, uuid: &Uuid) -> Option<*mut Self> {
        context.index.borrow().ir.get(uuid).map(|ptr| *ptr)
    }

    fn rooted(_: &Self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_ir() {
        let mut ctx = Context::new();
        let ir = IR::new(&mut ctx);
        assert_eq!(ir.version(), 1);
        assert_eq!(ir.modules().count(), 0);
    }

    #[test]
    fn new_ir_is_unique() {
        let mut ctx = Context::new();
        assert_ne!(IR::new(&mut ctx), IR::new(&mut ctx));
    }

    #[test]
    fn can_set_version() {
        let mut ctx = Context::new();
        let mut ir = IR::new(&mut ctx);
        ir.set_version(42);
        assert_eq!(ir.version(), 42);
    }

    #[test]
    fn can_add_new_module() {
        let mut ctx = Context::new();
        let mut ir = IR::new(&mut ctx);
        let module = Module::new(&mut ctx, "dummy");
        ir.add_module(module);

        let module = ir.modules().nth(0);
        assert!(module.is_some());
        assert_eq!(module.unwrap().ir().unwrap().uuid(), ir.uuid());
    }

    #[test]
    fn can_remove_module() {
        let mut ctx = Context::new();
        let mut ir = IR::new(&mut ctx);

        let module = Module::new(&mut ctx, "dummy");
        let uuid = module.uuid();
        ir.add_module(module);

        {
            let _module = ir.remove_module(uuid);
            assert_eq!(ir.modules().count(), 0);
        }

        // Module should be dropped after preceding scope.
        let node = ctx.find_node::<Module>(&uuid);
        assert!(node.is_none());
    }

    #[test]
    fn can_modify_modules() {
        let mut ctx = Context::new();
        let mut ir = IR::new(&mut ctx);
        ir.add_module(Module::new(&mut ctx, "foo"));
        ir.add_module(Module::new(&mut ctx, "bar"));
        for mut module in ir.modules() {
            module.set_preferred_address(Addr(1));
        }
        assert!(ir.modules().all(|m| m.preferred_address() == 1.into()));
    }
}
