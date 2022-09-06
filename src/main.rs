#[macro_use]
mod codegen;
mod utils;
mod log;
mod elf;


use codegen::CodeGen;

mod ir;
use ir::IR;
use elf::{assemble,  generate_elf};



#[cfg(feature = "riscv32")]
#[path = "arch/riscv32.rs"]
mod arch;


#[cfg(feature = "riscv64")]
#[path = "arch/riscv64.rs"]
mod arch;

fn main() {
    let ir = IR::new("unary.bc");
    let mut program = ir.parse("testcases/unary.S");
    program.codegen();
    assemble(
        "testcases/test.S",
        "testcases/test.o"
    );
    generate_elf(
        "testcases/test.o", 
        "testcases/test"
    );
}