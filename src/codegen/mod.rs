mod program;
mod var;
mod func;
use llvm_ir::Name;
pub use program::Program;
pub use func::{ Function, FuncParameter, FuncLocal, LocalValue };
pub use var::{ Var, Ty, VarValue };

pub enum ConstValue {
    Num(usize)
}

pub enum Op {
    ConstValue(ConstValue),
    LocalValue(Name)
}

pub trait CodeGen {
    fn emit_text(&mut self);
    fn emit_data(&mut self);
    fn codegen(&mut self);
}