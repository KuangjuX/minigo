mod program;
mod var;
mod func;
pub use program::Program;
pub use func::Function;
pub use var::{ Var, Ty, VarValue };

pub trait CodeGen {
    fn emit_text(&mut self);
    fn emit_data(&mut self);
    fn codegen(&mut self);
}