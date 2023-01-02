#[macro_use]
mod codegen;
mod utils;
mod log;
mod elf;


use codegen::CodeGen;

mod ir;
use ir::IR;
use elf::{assemble,  generate_elf, run_elf};
use std::env;



#[cfg(feature = "riscv32")]
#[path = "arch/riscv32.rs"]
mod arch;


#[cfg(feature = "riscv64")]
#[path = "arch/riscv64.rs"]
mod arch;

// static PROG: &'static str = env!("PROG");

fn main() {
    // let PROG = env!("PROG");
    let args: Vec<String> = env::args().collect();
    let prog = &args[1];
    let bc = format!("{}.bc", prog);
    let input = format!("{}.S", prog);
    let obj = format!("{}.o", prog);
    let elf = format!("{}.exe", prog);
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
    run_elf(elf.as_str());
}