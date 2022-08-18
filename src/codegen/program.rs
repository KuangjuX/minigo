use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use super::riscv32::Instruction;

use super::CodeGen;

pub struct Function {
    pub(crate) name: String,
    pub(crate) args: usize,
    pub(crate) is_static: bool,
    #[cfg(feature = "riscv32")]
    pub(crate) insts: VecDeque<Instruction>,
    #[cfg(not(feature = "riscv32"))]
    pub(crate) insts: VecDeque<Instruction>,
    pub(crate) stack_size: usize
}

pub struct Program {
    asm_file: File,
    funcs: VecDeque<Function>
}

impl Program {
    pub fn new(asm: File) -> Self {
        Self{
            asm_file: asm,
            funcs: VecDeque::new()
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