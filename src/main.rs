mod codegen;
mod utils;
mod log;
mod linker;
use codegen::CodeGen;

mod ir;
use ir::IR;
use linker::{assemble, run_linker};

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
    assemble();
    run_linker();
}