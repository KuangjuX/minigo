use llvm_ir::Instruction;
use std::collections::VecDeque;

/// program function define
pub struct Function {
    /// function name
    pub(crate) name: String,
    /// function is static
    pub(crate) is_static: bool,
    pub(crate) insts: VecDeque<Instruction>,
    /// function stack size
    pub(crate) stack_size: usize
}