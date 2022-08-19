use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::cell::RefCell;

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
    pub(crate) asm_file: RefCell<File>,
    pub(crate) funcs: VecDeque<Function>,
    pub(crate) vars: VecDeque<Var>
}

impl Program {
    pub fn new(asm: File) -> Self {
        Self{
            asm_file: RefCell::new(asm),
            funcs: VecDeque::new(),
            vars: VecDeque::new()
        }
    }

    pub fn debug(&self) {
        for var in self.vars.iter() {
            println!("var: {:?}", var);
        }
    }

    fn write_asm<S>(&self, asm: S) where S: Into<String> {
        let asm = format!("{}\n", asm.into());
        let mut asm_file = self.asm_file.borrow_mut();
        asm_file.write(asm.as_bytes()).unwrap();
    }

}

impl CodeGen for Program {
    fn emit_text(&mut self) {        
        // generate section
        for func in self.funcs.iter() {
            if func.is_static {
                self.write_asm("    .local");
            }else{
                self.write_asm("    .globl");
            }
            self.write_asm("    .text");
            let name = format!("{}:\n", func.name);
            self.write_asm(name);
        }

        // push all arguments into stack
    }

    fn emit_data(&mut self) {
        for var in self.vars.iter() {
            if var.is_static {
                let line = format!("    .local {}", var.name);
                self.write_asm(line);
            }else{
                let line = format!("    .globl {}", var.name);
                self.write_asm(line);
            }

            // .data or .tdata
            if var.initiazed {
                self.write_asm("    .data");
                let ty = format!("    .type {}, object", var.name);
                self.write_asm(ty);
                let size = format!("    .size {} {}", var.name, var.size);
                self.write_asm(size);
                let align = format!("    .align {}", var.align);
                self.write_asm(align);
                let name = format!("{}:", var.name);
                self.write_asm(name);
            }else {
                // .bss or .tbss
                self.write_asm("    .bss");
                let align = format!("    .align {}", var.align);
                self.write_asm(align);
                let name = format!("{}:", var.name);
                self.write_asm(name);
                let zero = format!("    .zero {}", var.ty.size());
            }

        }
    }

    fn codegen(&mut self) {
        self.emit_text();
        self.emit_data();
    }
}