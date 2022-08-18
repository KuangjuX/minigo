use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;

use super::CodeGen;

pub struct Function {

}

pub struct Program {
    asm_file: File
}

impl Program {
    pub fn new(asm: File) -> Self {
        Self{
            asm_file: asm
        }
    }

}

impl CodeGen for Program {
    fn emit_text(&mut self) {
        self.asm_file.write("   .global main\n".as_bytes()).unwrap();
        self.asm_file.write("main:\n".as_bytes()).unwrap();
        self.asm_file.write("addi t0, t0, 10\n".as_bytes()).unwrap();
        self.asm_file.write("ret\n".as_bytes()).unwrap();
    }

    fn codegen(&mut self) {
        self.emit_text();
    }
}