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

    fn test2() {
        let mut ir = crate::ir::IR::new();
        let mut a_module = ir.add_module("foo.exe");
        let a_section = a_module.add_section(&mut ir);

        for sec in a_module.sections_mut(&mut ir) {
            sec.set_name(&mut ir, String::from("bar"));
        }
    }
}
