use llvm_ir::{BasicBlock, name::Name};
use std::collections::VecDeque;

use crate::ir::VirtualReg;

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

    /// check if local variable exist in function
    pub fn local_var_exist(&self, name: Name) -> bool {
        for local_var in self.locals.iter() {
            if let Some(var_name) = local_var.name.clone() {
                if var_name == name {
                    return true
                }
            }
        }
        false
    }

    pub fn find_local_var(&self, name: Name) -> Option<&VirtualReg> {
        for local_var in self.locals.iter() {
            if let Some(var_name) = local_var.name.clone() {
                if var_name == name {
                    return local_var.local_val.as_ref()
                }
            }
        }
        None
    }
}