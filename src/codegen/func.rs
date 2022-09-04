use llvm_ir::{BasicBlock};
use std::collections::VecDeque;

use super::{Ty, Var};


/// program function define
pub struct Function {
    /// function name
    pub(crate) name: String,
    /// function is static
    pub(crate) is_static: bool,
    /// function basicblocks
    pub(crate) blocks: VecDeque<BasicBlock>,
    /// function stack size
    pub(crate) stack_size: usize,
    /// function params
    pub(crate) params: Vec<Var>,
    /// function local variables
    pub(crate) locals: Vec<Var>,
    /// function return type
    pub(crate) ret_ty: Ty
}

impl Function {
    pub fn uninit() -> Self {
        Self {
            name: String::new(),
            is_static: false,
            blocks: VecDeque::new(),
            stack_size: 0,
            params: Vec::new(),
            locals: Vec::new(),
            ret_ty: Ty::Unknown
        }
    }
}