use llvm_ir::{Instruction, BasicBlock};
use std::collections::VecDeque;

use super::Ty;


pub struct FuncParameter {
    pub ty: Ty,
    pub size: usize
}

/// program function define
pub struct Function {
    /// function name
    pub(crate) name: String,
    /// function is static
    pub(crate) is_static: bool,
    /// function basicblocks
    pub(crate) insts: VecDeque<BasicBlock>,
    /// function stack size
    pub(crate) stack_size: usize,
    /// function params
    pub(crate) params: Vec<FuncParameter>,
    /// function return type
    pub(crate) ret_ty: Ty
}

impl Function {
    pub fn uninit() -> Self {
        Self {
            name: String::new(),
            is_static: false,
            insts: VecDeque::new(),
            stack_size: 0,
            params: Vec::new(),
            ret_ty: Ty::Unknown
        }
    }
}