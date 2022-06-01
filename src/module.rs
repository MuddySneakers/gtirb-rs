use anyhow::{anyhow, Result};

use crate::*;

#[derive(Debug, Default, PartialEq)]
pub struct ModuleData {
    pub(crate) parent: Option<Index>,

    uuid: Uuid,
    name: String,
    binary_path: String,
    entry_point: Option<Uuid>,
    byte_order: ByteOrder,
    isa: ISA,
    rebase_delta: i64,
    preferred_address: Addr,
    file_format: FileFormat,
    sections: Vec<Index>,
    symbols: Vec<Index>,
    proxy_blocks: Vec<Index>,
}

impl ModuleData {
    pub fn new(name: &str) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: name.to_owned(),
            ..Default::default()
        }
    }

    pub(crate) fn load_protobuf(
        context: Rc<RefCell<Context>>,
        message: proto::Module,
    ) -> Result<Index> {
        let format = FileFormat::from_i32(message.file_format)
            .ok_or(anyhow!("Invalid FileFormat"))?;

        let isa = ISA::from_i32(message.isa).ok_or(anyhow!("Invalid ISA"))?;

        let byte_order = ByteOrder::from_i32(message.byte_order)
            .ok_or(anyhow!("Invalid ByteOrder"))?;

        let sections = message
            .sections
            .into_iter()
            .map(|m| SectionData::load_protobuf(context.clone(), m))
            .collect::<Result<Vec<Index>>>()?;

        let symbols = message
            .symbols
            .into_iter()
            .map(|m| SymbolData::load_protobuf(context.clone(), m))
            .collect::<Result<Vec<Index>>>()?;

        let proxy_blocks = message
            .proxies
            .into_iter()
            .map(|m| ProxyBlockData::load_protobuf(context.clone(), m))
            .collect::<Result<Vec<Index>>>()?;

        let module = Self {
            parent: None,

            uuid: crate::util::parse_uuid(&message.uuid)?,
            name: message.name,
            binary_path: message.binary_path,
            entry_point: Some(crate::util::parse_uuid(&message.entry_point)?),
            byte_order: byte_order,
            isa: isa,
            rebase_delta: message.rebase_delta,
            preferred_address: Addr(message.preferred_addr),
            file_format: format,
            sections: sections,
            symbols: symbols,
            proxy_blocks: proxy_blocks,
        };

        Ok(context.borrow_mut().module.insert(module))
    }
}

impl NodeData<Module> for ModuleData {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    index: Index,
    context: Rc<RefCell<Context>>,
}

impl Module {
    pub fn ir(&self) -> IR {
        IR::new(
            self.borrow().parent.expect("parent node"),
            self.context.clone(),
        )
    }

    pub fn name(&self) -> String {
        self.borrow().name.to_owned()
    }

    pub fn set_name<T: AsRef<str>>(&self, name: T) {
        self.borrow_mut().name = name.as_ref().to_owned();
    }

    pub fn binary_path(&self) -> String {
        self.borrow().binary_path.to_owned()
    }

    pub fn set_binary_path<T: AsRef<str>>(&self, binary_path: T) {
        self.borrow_mut().binary_path = binary_path.as_ref().to_owned();
    }

    pub fn file_format(&self) -> FileFormat {
        self.borrow().file_format
    }

    pub fn set_file_format(&self, file_format: FileFormat) {
        self.borrow_mut().file_format = file_format;
    }

    pub fn isa(&self) -> ISA {
        self.borrow().isa
    }

    pub fn set_isa(&self, isa: ISA) {
        self.borrow_mut().isa = isa;
    }

    pub fn entry_point(&self) -> Option<CodeBlock> {
        self.borrow()
            .entry_point
            .and_then(|uuid| self.ir().find_node(uuid))
    }

    pub fn set_entry_point(&self, block: CodeBlock) {
        self.borrow_mut().entry_point.replace(block.uuid());
    }

    pub fn byte_order(&self) -> ByteOrder {
        self.borrow().byte_order
    }

    pub fn set_byte_order(&self, byte_order: ByteOrder) {
        self.borrow_mut().byte_order = byte_order;
    }

    pub fn preferred_address(&self) -> Addr {
        self.borrow().preferred_address
    }

    pub fn set_preferred_address(&self, preferred_address: Addr) {
        self.borrow_mut().preferred_address = preferred_address;
    }

    pub fn rebase_delta(&self) -> i64 {
        self.borrow().rebase_delta
    }

    pub fn set_rebase_delta(&self, rebase_delta: i64) {
        self.borrow_mut().rebase_delta = rebase_delta;
    }

    pub fn is_relocated(&self) -> bool {
        self.borrow().rebase_delta != 0
    }

    pub fn sections(&self) -> NodeIterator<Section, SectionData> {
        self.node_iter()
    }

    pub fn add_section(&self, section: SectionData) -> Section {
        self.add_node(section)
    }

    pub fn remove_section(&self, node: Section) {
        self.remove_node(node);
    }

    pub fn proxy_blocks(&self) -> NodeIterator<ProxyBlock, ProxyBlockData> {
        self.node_iter()
    }

    pub fn add_proxy_block(&self, proxy_block: ProxyBlockData) -> ProxyBlock {
        self.add_node(proxy_block)
    }

    pub fn remove_proxy_block(&self, node: ProxyBlock) {
        self.remove_node(node);
    }

    pub fn symbols(&self) -> NodeIterator<Symbol, SymbolData> {
        self.node_iter()
    }

    pub fn add_symbol(&self, symbol: SymbolData) -> Symbol {
        self.add_node(symbol)
    }

    pub fn remove_symbol(&self, node: Symbol) {
        self.remove_node(node);
    }

    pub fn size(&self) -> Option<u64> {
        let min: Option<Addr> =
            self.sections().map(|i| i.address()).min().flatten();
        let max: Option<Addr> = self
            .sections()
            .map(|i| {
                i.address()
                    .zip(i.size())
                    .map(|(addr, size)| addr + size.into())
            })
            .max()
            .flatten();
        if let (Some(min), Some(max)) = (min, max) {
            Some(u64::from(max - min))
        } else {
            None
        }
    }

    pub fn address(&self) -> Option<Addr> {
        self.sections().map(|s| s.address()).min().flatten()
    }

    pub fn byte_intervals(
        &self,
    ) -> NodeIterator<ByteInterval, ByteIntervalData> {
        let iter = self.sections().flat_map(|interval| {
            <Section as Parent<ByteInterval, ByteIntervalData>>::nodes(
                &interval,
            )
            .clone()
            .into_iter()
        });
        NodeIterator {
            iter: Box::new(iter),
            context: self.context.clone(),
            kind: PhantomData,
            kind_data: PhantomData,
        }
    }

    pub fn code_blocks(&self) -> NodeIterator<CodeBlock, CodeBlockData> {
        let iter = self.sections().flat_map(|section| {
            section.byte_intervals().flat_map(|interval| {
                <ByteInterval as Parent<CodeBlock, CodeBlockData>>::nodes(
                    &interval,
                )
                .clone()
                .into_iter()
            })
        });
        NodeIterator {
            iter: Box::new(iter),
            context: self.context.clone(),
            kind: PhantomData,
            kind_data: PhantomData,
        }
    }

    pub fn data_blocks(&self) -> NodeIterator<DataBlock, DataBlockData> {
        let iter = self.sections().flat_map(|section| {
            section.byte_intervals().flat_map(|interval| {
                <ByteInterval as Parent<DataBlock, DataBlockData>>::nodes(
                    &interval,
                )
                .clone()
                .into_iter()
            })
        });
        NodeIterator {
            iter: Box::new(iter),
            context: self.context.clone(),
            kind: PhantomData,
            kind_data: PhantomData,
        }
    }

    // symbolic_expressions()
    // get_symbol_reference<T>(symbol: Symbol) -> Node<T>
}

impl Node<Module, ModuleData> for Module {
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

    fn arena(&self) -> Ref<Arena<ModuleData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.module)
    }

    fn arena_mut(&self) -> RefMut<Arena<ModuleData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.module)
    }
}

impl Child<IR, IRData> for Module {
    fn parent(&self) -> (Option<Index>, PhantomData<IR>) {
        (self.borrow().parent, PhantomData)
    }

    fn set_parent(&self, (index, _): (Index, PhantomData<IR>)) {
        self.borrow_mut().parent.replace(index);
    }
}

impl Parent<Section, SectionData> for Module {
    fn nodes(&self) -> Ref<Vec<Index>> {
        Ref::map(self.borrow(), |module: &ModuleData| &module.sections)
    }

    fn nodes_mut(&self) -> RefMut<Vec<Index>> {
        RefMut::map(self.borrow_mut(), |module: &mut ModuleData| {
            &mut module.sections
        })
    }

    fn child_arena(&self) -> Ref<Arena<SectionData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.section)
    }

    fn child_arena_mut(&self) -> RefMut<Arena<SectionData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.section)
    }
}

impl Parent<ProxyBlock, ProxyBlockData> for Module {
    fn nodes(&self) -> Ref<Vec<Index>> {
        Ref::map(self.borrow(), |module: &ModuleData| &module.proxy_blocks)
    }

    fn nodes_mut(&self) -> RefMut<Vec<Index>> {
        RefMut::map(self.borrow_mut(), |module: &mut ModuleData| {
            &mut module.proxy_blocks
        })
    }

    fn child_arena(&self) -> Ref<Arena<ProxyBlockData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.proxy_block)
    }

    fn child_arena_mut(&self) -> RefMut<Arena<ProxyBlockData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.proxy_block)
    }
}

impl Parent<Symbol, SymbolData> for Module {
    fn nodes(&self) -> Ref<Vec<Index>> {
        Ref::map(self.borrow(), |module: &ModuleData| &module.symbols)
    }

    fn nodes_mut(&self) -> RefMut<Vec<Index>> {
        RefMut::map(self.borrow_mut(), |module: &mut ModuleData| {
            &mut module.symbols
        })
    }

    fn child_arena(&self) -> Ref<Arena<SymbolData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.symbol)
    }

    fn child_arena_mut(&self) -> RefMut<Arena<SymbolData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.symbol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_module_is_unique() {
        assert_ne!(ModuleData::new("a"), ModuleData::new("a"));
    }

    #[test]
    fn new_module_is_empty() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("dummy"));

        assert_eq!(module.symbols().count(), 0);
        assert_eq!(module.sections().count(), 0);
        assert_eq!(module.proxy_blocks().count(), 0);
    }

    #[test]
    fn can_set_binary_path() {
        let ir = IRData::new();
        let path = "/home/gt/irb/foo";
        let module = ir.add_module(ModuleData::new("dummy"));
        module.set_binary_path(path);
        assert_eq!(module.binary_path(), path);
    }

    #[test]
    fn can_get_file_format_default() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("dummy"));
        assert_eq!(module.file_format(), FileFormat::FormatUndefined);
    }

    #[test]
    fn can_set_file_format() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("dummy"));
        module.set_file_format(FileFormat::Coff);
        assert_eq!(module.file_format(), FileFormat::Coff);

        module.set_file_format(FileFormat::Macho);
        assert_eq!(module.file_format(), FileFormat::Macho);
    }

    #[test]
    fn can_set_name() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("dummy"));
        module.set_name("example");
        assert_eq!(module.name(), "example");
    }

    #[test]
    fn can_relocate_module() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("dummy"));
        assert!(!module.is_relocated());
        assert_eq!(module.rebase_delta(), 0);

        module.set_rebase_delta(0x1000);
        assert!(module.is_relocated());
        assert_eq!(module.rebase_delta(), 0x1000);
    }

    #[test]
    fn can_add_new_section() {
        let ir = IRData::new();
        let module = ModuleData::new("dummy");
        let module = ir.add_module(module);
        assert_eq!(module.ir(), ir);
    }

    #[test]
    fn can_remove_section() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("foo"));
        let section = module.add_section(SectionData::new("bar"));
        module.remove_section(section);
        assert_eq!(module.sections().count(), 0);
    }

    #[test]
    fn can_iterate_over_code_blocks() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("dummy"));
        let section = module.add_section(SectionData::new(".dummy"));
        let b1 = section.add_byte_interval(ByteIntervalData::new());
        let b2 = section.add_byte_interval(ByteIntervalData::new());
        let cb1 = b1.add_code_block(CodeBlockData::new());
        let cb2 = b2.add_code_block(CodeBlockData::new());
        assert_eq!(
            module
                .code_blocks()
                .map(|cb| cb.uuid())
                .collect::<Vec<Uuid>>(),
            vec![cb1.uuid(), cb2.uuid()]
        );
        assert_eq!(
            section
                .code_blocks()
                .map(|cb| cb.uuid())
                .collect::<Vec<Uuid>>(),
            vec![cb1.uuid(), cb2.uuid()]
        );
    }

    #[test]
    fn can_calculate_size() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("dummy"));
        assert_eq!(module.size(), None);
        assert_eq!(module.address(), None);

        let text = module.add_section(SectionData::new(".text"));
        let bytes = text.add_byte_interval(ByteIntervalData::new());
        bytes.set_address(Some(Addr(200)));
        bytes.set_size(100);

        assert!(module.address().is_some());
        assert_eq!(module.size(), Some(100));
        assert_eq!(module.address(), Some(Addr(200)));

        bytes.set_address(Some(Addr(0)));
        assert_eq!(module.address(), Some(Addr(0)));

        let data = module.add_section(SectionData::new(".data"));
        let bytes = data.add_byte_interval(ByteIntervalData::new());
        bytes.set_address(Some(Addr(300)));
        bytes.set_size(100);
        assert_eq!(module.size(), Some(400));
        assert_eq!(module.address(), Some(Addr(0)));

        assert_eq!(module.byte_intervals().count(), 2);
    }
}
