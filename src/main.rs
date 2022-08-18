use std::fs::File;
mod codegen;
use codegen::CodeGen;

fn main() {
    let out_asm = File::create("main.S").unwrap();
    let mut program = codegen::Program::new(out_asm);
    program.codegen();
}