use uuid::Uuid;

mod ir;
mod module;
mod section;
mod byte_interval;
mod code_block;
mod data_block;
mod symbolic_expression;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test1() {
        let mut ir = crate::ir::IR::new();
        let a_module = ir.add_module("foo.exe");
        assert_eq!(a_module.get_name(&ir), "foo.exe");
    }
}
