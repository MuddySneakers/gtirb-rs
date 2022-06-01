use std::collections::HashSet;

use anyhow::{anyhow, Result};

use crate::*;

#[derive(Default, Debug, PartialEq)]
pub struct SectionData {
    pub(crate) parent: Option<Index>,

    uuid: Uuid,
    name: String,
    flags: HashSet<SectionFlag>,
    byte_intervals: Vec<Index>,
}

impl SectionData {
    pub fn new(name: &str) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name: name.to_owned(),
            ..Default::default()
        }
    }

    pub(crate) fn load_protobuf(
        context: Rc<RefCell<Context>>,
        message: proto::Section,
    ) -> Result<Index> {
        let section_flags: Result<HashSet<SectionFlag>> = message
            .section_flags
            .into_iter()
            .map(|i| {
                SectionFlag::from_i32(i).ok_or(anyhow!("Invalid FileFormat"))
            })
            .collect();

        let byte_intervals = message
            .byte_intervals
            .into_iter()
            .map(|m| ByteIntervalData::load_protobuf(context.clone(), m))
            .collect::<Result<Vec<Index>>>()?;

        let section = Self {
            parent: None,

            uuid: crate::util::parse_uuid(&message.uuid)?,
            name: message.name,
            flags: section_flags?,
            byte_intervals: byte_intervals,
        };

        Ok(context.borrow_mut().section.insert(section))
    }
}

impl NodeData<Section> for SectionData {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

#[derive(Clone, Debug)]
pub struct Section {
    index: Index,
    context: Rc<RefCell<Context>>,
}

impl Section {
    pub fn name(&self) -> String {
        self.borrow().name.to_owned()
    }

    pub fn set_name<T: AsRef<str>>(&self, name: T) {
        self.borrow_mut().name = name.as_ref().to_owned();
    }

    pub fn flags(&self) -> HashSet<SectionFlag> {
        self.borrow().flags.clone()
    }

    pub fn add_flag(&self, flag: SectionFlag) {
        self.borrow_mut().flags.insert(flag);
    }

    pub fn remove_flag(&self, flag: SectionFlag) {
        self.borrow_mut().flags.remove(&flag);
    }

    pub fn is_flag_set(&self, flag: SectionFlag) -> bool {
        self.borrow().flags.contains(&flag)
    }

    pub fn size(&self) -> Option<u64> {
        let min: Option<Addr> =
            self.byte_intervals().map(|i| i.address()).min().flatten();
        let max: Option<Addr> = self
            .byte_intervals()
            .map(|i| i.address().map(|a| a + i.size().into()))
            .max()
            .flatten();
        if let (Some(min), Some(max)) = (min, max) {
            Some(u64::from(max - min))
        } else {
            None
        }
    }

    pub fn address(&self) -> Option<Addr> {
        self.byte_intervals().map(|i| i.address()).min().flatten()
    }

    pub fn byte_intervals(
        &self,
    ) -> NodeIterator<ByteInterval, ByteIntervalData> {
        self.node_iter()
    }

    pub fn add_byte_interval(
        &self,
        byte_interval: ByteIntervalData,
    ) -> ByteInterval {
        self.add_node(byte_interval)
    }

    pub fn remove_byte_interval(&self, node: ByteInterval) {
        self.remove_node(node);
    }

    pub fn code_blocks(&self) -> NodeIterator<CodeBlock, CodeBlockData> {
        let iter = self.byte_intervals().flat_map(|interval| {
            <ByteInterval as Parent<CodeBlock, CodeBlockData>>::nodes(&interval)
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

    pub fn data_blocks(&self) -> NodeIterator<DataBlock, DataBlockData> {
        let iter = self.byte_intervals().flat_map(|interval| {
            <ByteInterval as Parent<DataBlock, DataBlockData>>::nodes(&interval)
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
}

impl Node<Section, SectionData> for Section {
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

    fn arena(&self) -> Ref<Arena<SectionData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.section)
    }

    fn arena_mut(&self) -> RefMut<Arena<SectionData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.section)
    }
}

impl Child<Module, ModuleData> for Section {
    fn parent(&self) -> (Option<Index>, PhantomData<Module>) {
        (self.borrow().parent, PhantomData)
    }

    fn set_parent(&self, (index, _): (Index, PhantomData<Module>)) {
        self.borrow_mut().parent.replace(index);
    }
}

impl Parent<ByteInterval, ByteIntervalData> for Section {
    fn nodes(&self) -> Ref<Vec<Index>> {
        Ref::map(self.borrow(), |section| &section.byte_intervals)
    }

    fn nodes_mut(&self) -> RefMut<Vec<Index>> {
        RefMut::map(self.borrow_mut(), |section| &mut section.byte_intervals)
    }

    fn child_arena(&self) -> Ref<Arena<ByteIntervalData>> {
        Ref::map(self.context.borrow(), |ctx| &ctx.byte_interval)
    }
    fn child_arena_mut(&self) -> RefMut<Arena<ByteIntervalData>> {
        RefMut::map(self.context.borrow_mut(), |ctx| &mut ctx.byte_interval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_set_attributes() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("dummy"));
        let section = module.add_section(SectionData::new(".text"));
        assert_eq!(section.name(), ".text");

        section.set_name(".data");
        assert_eq!(section.name(), ".data");
    }

    #[test]
    fn can_set_flags() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("dummy"));
        let section = module.add_section(SectionData::new(".text"));
        assert_eq!(section.name(), ".text");

        assert!(section.flags().is_empty());
        section.add_flag(SectionFlag::Readable);
        section.add_flag(SectionFlag::Writable);
        assert!(section.is_flag_set(SectionFlag::Readable));
        assert!(section.is_flag_set(SectionFlag::Writable));

        section.remove_flag(SectionFlag::Writable);
        assert!(!section.is_flag_set(SectionFlag::Writable));
    }

    #[test]
    fn can_calculate_size() {
        let ir = IRData::new();
        let module = ir.add_module(ModuleData::new("dummy"));

        let section = module.add_section(SectionData::new(".text"));
        assert_eq!(section.size(), None);
        assert_eq!(section.address(), None);

        let byte_interval = section.add_byte_interval(ByteIntervalData::new());
        byte_interval.set_address(Some(Addr(5)));
        byte_interval.set_size(10);
        assert_eq!(section.size(), Some(10));
        assert_eq!(section.address(), Some(Addr(5)));

        let byte_interval = section.add_byte_interval(ByteIntervalData::new());
        byte_interval.set_address(Some(Addr(15)));
        byte_interval.set_size(10);
        assert_eq!(section.size(), Some(20));
        assert_eq!(section.address(), Some(Addr(5)));

        section.add_byte_interval(ByteIntervalData::new());
        assert_eq!(section.size(), None);
        assert_eq!(section.address(), None);
    }
}
