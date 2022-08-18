mod arch;
use arch::riscv32;

mod program;

pub trait CodeGen {
    fn gen_asm();
}