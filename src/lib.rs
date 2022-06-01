#![allow(dead_code)]

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use generational_arena::{Arena, Index};
use uuid::Uuid;

mod proto {
    include!(concat!(env!("OUT_DIR"), "/gtirb.proto.rs"));
}

pub use proto::{ByteOrder, FileFormat, Isa as ISA, SectionFlag};

mod addr;
use addr::*;

mod ir;
pub use ir::read;
use ir::{IRData, IR};

mod module;
use module::{Module, ModuleData};

mod section;
use section::{Section, SectionData};

mod byte_interval;
use byte_interval::{ByteInterval, ByteIntervalData};

mod code_block;
use code_block::{CodeBlock, CodeBlockData};

mod data_block;
use data_block::{DataBlock, DataBlockData};

mod proxy_block;
use proxy_block::{ProxyBlock, ProxyBlockData};

mod symbol;
use symbol::{Symbol, SymbolData};

mod symbolic_expression;
use symbolic_expression::SymbolicExpression;

mod util;

pub trait NodeData<T> {
    fn uuid(&self) -> Uuid;
}

pub trait Node<T: Node<T, TData>, TData: NodeData<T>> {
    fn new(idx: Index, ctx: Rc<RefCell<Context>>) -> Self;
    fn index(&self) -> Index;
    fn context(&self) -> Rc<RefCell<Context>>;
    fn arena(&self) -> Ref<Arena<TData>>;
    fn arena_mut(&self) -> RefMut<Arena<TData>>;
    fn uuid(&self) -> Uuid;
    fn set_uuid(&mut self, uuid: Uuid);

    fn node_iter<U, UData>(&self) -> NodeIterator<U, UData>
    where
        U: Child<T, TData> + Node<U, UData>,
        UData: NodeData<U>,
        Self: Parent<U, UData>,
    {
        NodeIterator {
            iter: Box::new(self.nodes().clone().into_iter()),
            context: self.context().clone(),
            kind: PhantomData,
            kind_data: PhantomData,
        }
    }

    fn add_node<U, UData>(&self, node_data: UData) -> U
    where
        U: Child<T, TData> + Node<U, UData>,
        UData: NodeData<U>,
        Self: Parent<U, UData>,
    {
        // Add node to Context.
        let uuid = node_data.uuid();
        let index = self.child_arena_mut().insert(node_data);
        self.context().borrow_mut().uuid_map.insert(uuid, index);

        // Add node to Parent.
        self.nodes_mut().push(index);

        let node = U::new(index, self.context().clone());

        // Update parent
        node.set_parent((self.index(), PhantomData));
        node
    }

    fn remove_node<U, UData>(&self, node: U)
    where
        Self: Parent<U, UData>,
        U: Child<T, TData> + Node<U, UData>,
        UData: NodeData<U>,
    {
        // Consume node.
        let (index, uuid) = { (node.index(), node.uuid()) };

        // Remove Child from Parent.
        let position = self.nodes().iter().position(|i| *i == index).unwrap();
        self.nodes_mut().remove(position);

        // Remove Child from Context.
        self.child_arena_mut().remove(index);
        self.context().borrow_mut().uuid_map.remove(&uuid);
    }

    fn borrow(&self) -> Ref<TData> {
        Ref::map(self.arena(), |a| a.get(self.index()).expect("indexed node"))
    }

    fn borrow_mut(&self) -> RefMut<TData> {
        RefMut::map(self.arena_mut(), |a| {
            a.get_mut(self.index()).expect("indexed node")
        })
    }
}

pub struct NodeIterator<T, TData>
where
    T: Node<T, TData>,
    TData: NodeData<T>,
{
    iter: Box<dyn Iterator<Item = Index>>,
    context: Rc<RefCell<Context>>,
    kind: PhantomData<T>,
    kind_data: PhantomData<TData>,
}

impl<T, TData> Iterator for NodeIterator<T, TData>
where
    T: Node<T, TData>,
    TData: NodeData<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next();
        item.map(|index| T::new(index, self.context.clone()))
    }
}

pub trait Child<T, TData>
where
    T: Node<T, TData>,
    TData: NodeData<T>,
{
    fn parent(&self) -> (Option<Index>, PhantomData<T>);
    fn set_parent(&self, index: (Index, PhantomData<T>));
}

pub trait Parent<T, TData>
where
    T: Node<T, TData>,
    TData: NodeData<T>,
{
    fn nodes(&self) -> Ref<Vec<Index>>;
    fn nodes_mut(&self) -> RefMut<Vec<Index>>;

    fn child_arena(&self) -> Ref<Arena<TData>>;
    fn child_arena_mut(&self) -> RefMut<Arena<TData>>;
}

#[derive(Debug, Default)]
pub struct Context {
    uuid_map: HashMap<Uuid, Index>,

    ir: Arena<IRData>,
    module: Arena<ModuleData>,
    section: Arena<SectionData>,
    byte_interval: Arena<ByteIntervalData>,
    code_block: Arena<CodeBlockData>,
    data_block: Arena<DataBlockData>,
    proxy_block: Arena<ProxyBlockData>,
    symbol: Arena<SymbolData>,
}

impl Context {
    fn new() -> Self {
        Default::default()
    }
}
