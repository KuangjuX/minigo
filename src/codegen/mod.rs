use std::io::Result;

mod arch;
use arch::riscv32;

mod program;
pub use program::Program;

pub trait CodeGen {
    fn emit_text(&mut self);
    fn emit_data(&mut self);
    fn codegen(&mut self);
}