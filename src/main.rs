#[macro_use]
mod codegen;
mod utils;
mod log;
mod elf;


use codegen::CodeGen;

mod ir;
use ir::IR;
use elf::{assemble,  generate_elf};
use std::env;



#[cfg(feature = "riscv32")]
#[path = "arch/riscv32.rs"]
mod arch;


#[cfg(feature = "riscv64")]
#[path = "arch/riscv64.rs"]
mod arch;

// static PROG: &'static str = env!("PROG");

fn main() {
    let PROG = env!("PROG");
    let bc = format!("{}.bc", PROG);
    let input = format!("testcases/{}.S", PROG);
    let obj = format!("testcases/{}.o", PROG);
    let elf = format!("testcases/{}", PROG);
    let ir = IR::new(bc);
    let mut program = ir.parse(input.as_str());
    program.codegen();
    assemble(
        input.as_str(),
        obj.as_str()
    );
    generate_elf(
        obj.as_str(),
        elf.as_str()
    );
}