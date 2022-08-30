mod codegen;
mod utils;
mod log;
mod elf;
use codegen::CodeGen;

mod ir;
use ir::IR;
use elf::{assemble, run_linker};

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
    assemble(
        "testcases/test.S",
        "testcases/test.o"
    );
    run_linker(
        "testcases/test.S",
        "testcases/test"
    );
}