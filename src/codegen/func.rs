use llvm_ir::{BasicBlock, name::Name};
use std::{collections::VecDeque, cell::RefCell};

use crate::ir::{VirtualReg, RegVar};

use super::{Ty, Var};

/// BasicBlock label
#[derive(Clone)]
pub struct Label {
    pub(crate) llvm_name: Name,
    pub(crate) label_name: String
}


/// program function define
pub struct Function {
    /// function name
    pub(crate) name: String,
    /// function is static
    pub(crate) is_static: bool,
    /// function basicblocks
    pub(crate) blocks: VecDeque<BasicBlock>,
    /// function params
    pub(crate) params: Vec<Var>,
    /// function return type
    pub(crate) ret_ty: Ty,
    /// function inner, which will be changer during runtime
    pub(crate) inner: RefCell<FuncInner>,
}

pub struct FuncInner {
    /// function stack size
    pub(crate) stack_size: usize,
    /// function local variables
    pub(crate) locals: Vec<Var>,

    /// Label
    pub(crate) labels: Vec<Label>
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
                locals: Vec::new(),
                labels: Vec::new()
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

    /// push local variable into vec
    pub(crate) fn add_local_var(&self, var: Var) {
        let mut inner = self.inner.borrow_mut();
        inner.locals.push(var);
    }

    /// remove local variable in vec
    pub(crate) fn remove_local_var(&self, name: Name) {
        let mut inner = self.inner.borrow_mut();
        if let Some(index) = inner.locals.iter().position(|item| item.name == Some(name.clone())){
            inner.locals.remove(index);
        }
    }

    pub(crate) fn find_label(&self, name: Name) -> Option<Label> {
        let mut inner = self.inner.borrow();
        for item in inner.labels.iter() {
            if item.llvm_name == name {
                return Some(item.clone())
            }
        }
        None
    }
}