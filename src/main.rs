use std::fs::File;
mod codegen;
use codegen::CodeGen;

mod ir;
use ir::IR;
use llvm_ir::Module;

use crate::codegen::Program;

#[cfg(feature = "riscv32")]
#[path = "arch/riscv32.rs"]
mod arch;


#[cfg(feature = "riscv64")]
#[path = "arch/riscv64.rs"]
mod arch;

fn main() {
    let mut ir = IR::new("test.bc");
    let mut program = ir.parse();
    program.codegen();
    
}