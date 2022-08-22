mod program;
mod var;
pub use program::{ Program, Function };
pub use var::{ Var, Ty };

pub trait CodeGen {
    fn emit_text(&mut self);
    fn emit_data(&mut self);
    fn codegen(&mut self);
}