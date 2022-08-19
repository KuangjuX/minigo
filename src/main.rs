use std::fs::File;
mod codegen;
use codegen::CodeGen;

mod ir;
use ir::IR;

#[cfg(feature = "riscv32")]
#[path = "arch/riscv32.rs"]
mod arch;


#[cfg(feature = "riscv64")]
#[path = "arch/riscv64.rs"]
mod arch;

fn main() {
    let ll = File::open("test.ll").unwrap();
    let mut ir = IR::new(ll);
    let mut program = ir.parse();
    
    // let out_asm = File::create("main.S").unwrap();
    // let mut program = codegen::Program::new(out_asm);
    program.debug();
    program.codegen();
}