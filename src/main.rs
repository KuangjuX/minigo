mod codegen;
mod utils;
mod log;
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
    let ir = IR::new("test.bc");
    let mut program = ir.parse();
    program.codegen();
    
}