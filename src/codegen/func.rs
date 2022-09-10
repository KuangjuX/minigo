use llvm_ir::{BasicBlock, name::Name};
use std::{collections::VecDeque, cell::RefCell};

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
    // pub(crate) stack_size: usize,
    /// function params
    pub(crate) params: Vec<Var>,
    /// function local variables
    // pub(crate) locals: Vec<Var>,
    /// function return type
    pub(crate) ret_ty: Ty,

    pub(crate) inner: RefCell<FuncInner>
}

pub struct FuncInner {
    pub(crate) stack_size: usize,
    pub(crate) locals: Vec<Var>
}

impl Function {
    pub fn uninit() -> Self {
        Self {
            name: String::new(),
            is_static: false,
            blocks: VecDeque::new(),
            params: Vec::new(),
            ret_ty: Ty::Unknown,
            inner: RefCell::new(FuncInner{ 
                stack_size: 0,
                locals: Vec::new()
            })
        }
    }

    /// check if local variable exist in function
    pub fn local_var_exist(&self, name: Name) -> bool {
        let inner = self.inner.borrow();
        for local_var in inner.locals.iter() {
            if let Some(var_name) = local_var.name.clone() {
                if var_name == name {
                    return true
                }
            }
        }
        false
    }

    pub fn find_local_var(&self, name: Name) -> Option<VirtualReg> {
        let inner = self.inner.borrow();
        for local_var in inner.locals.iter() {
            if let Some(var_name) = local_var.name.clone() {
                if var_name == name {
                    return local_var.local_val.clone()
                }
            }
        }
        None
    }

    pub fn push_var(&self, size: usize) {
        let mut func_inner = self.inner.borrow_mut();
        func_inner.stack_size += size;
    }

    pub fn stack_size(&self) -> usize {
        let inner = self.inner.borrow();
        inner.stack_size
    }

    pub(crate) fn add_local_var(&self, var: Var) {
        let mut inner = self.inner.borrow_mut();
        inner.locals.push(var);
    }
}