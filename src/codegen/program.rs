use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use crate::arch::Instruction;

use super::{CodeGen, Var};

pub struct Function {
    pub(crate) name: String,
    pub(crate) args: usize,
    pub(crate) is_static: bool,
    pub(crate) insts: VecDeque<Instruction>,
    pub(crate) stack_size: usize
}

pub struct Program {
    pub(crate) asm_file: File,
    pub(crate) funcs: VecDeque<Function>,
    pub(crate) vars: VecDeque<Var>
}

impl Program {
    pub fn new(asm: File) -> Self {
        Self{
            asm_file: asm,
            funcs: VecDeque::new(),
            vars: VecDeque::new()
        }
    }

    pub fn debug(&self) {
        for var in self.vars.iter() {
            println!("var: {:?}", var);
        }
    }

}

impl CodeGen for Program {
    fn emit_text(&mut self) {        
        // generate section
        for func in self.funcs.iter() {
            if func.is_static {
                self.asm_file.write("   .local\n".as_bytes()).unwrap();
            }else{
                self.asm_file.write("   .globl\n".as_bytes()).unwrap();
            }
            self.asm_file.write(".text\n".as_bytes()).unwrap();
            let name = format!("{}:\n", func.name);
            self.asm_file.write(name.as_bytes()).unwrap();
        }

        // push all arguments into stack
    }

    fn emit_data(&mut self) {

    }

    fn codegen(&mut self) {
        self.emit_text();
        self.emit_data();
    }
}