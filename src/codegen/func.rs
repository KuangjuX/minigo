use llvm_ir::{BasicBlock, name::Name};
use std::{collections::VecDeque, cell::RefCell};

use crate::ir::{VirtualReg};

use super::{Ty, Var, Program};

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
    /// function param variables
    pub(crate) param_vars: Vec<Var>,
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
                param_vars: Vec::new(),
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

    pub fn get_reg_nums(&self) -> usize {
        let mut num = 0;
        let inner = self.inner.borrow();
        for local_var in inner.locals.iter() {
            if let Some(var) = local_var.name.clone() {
                let virt_reg = self.find_local_var(var).unwrap();
                match virt_reg {
                    VirtualReg::Reg(reg) => { num += 1}
                    _ => {}
                }
            }
        }
        num
    }

    /// 保存寄存器上下文
    pub fn store_regs(&self, prog: &Program) {
        let mut index = 0;
        let reg_nums = self.get_reg_nums();
        let stack_size = 8 * reg_nums;
        let asm = format!("\taddi sp, sp, -{}", stack_size);
        prog.write_asm(asm);
        // 对使用过的寄存器进行保存
        let inner = self.inner.borrow();
        for var in inner.locals.iter() {
            if let Some(name) = &var.name {
                let virt_reg = self.find_local_var(name.clone()).unwrap();
                match virt_reg {
                    VirtualReg::Reg(reg_var) => {
                        let asm = format!("\tsd {}, {}(sp)", reg_var.name, index);
                        prog.write_asm(asm);
                        index += 8;
                    },
                    _ => {}
                }
            }
        }
    }

    /// 恢复寄存器上下文
    pub fn restore_regs(&self, prog: &Program) {
        let mut index = 0;
        let reg_nums = self.get_reg_nums();
        let stack_size = 8 * reg_nums;
        // 对使用过的寄存器进行保存
        let inner = self.inner.borrow();
        for var in inner.locals.iter() {
            if let Some(name) = &var.name {
                let virt_reg = self.find_local_var(name.clone()).unwrap();
                match virt_reg {
                    VirtualReg::Reg(reg_var) => {
                        let asm = format!("\tld {}, {}(sp)", reg_var.name, index);
                        prog.write_asm(asm);
                        index += 8;
                    },
                    _ => {}
                }
            }
        }
        let asm = format!("\taddi sp, sp, {}", stack_size);
        prog.write_asm(asm);
    }

    /// 找到对应的局部变量
    pub fn find_local_var(&self, name: Name) -> Option<VirtualReg> {
        let inner = self.inner.borrow();
        for local_var in inner.locals.iter() {
            if let Some(var_name) = local_var.name.clone() {
                if var_name == name {
                    return local_var.local_val.clone()
                }
            }
        }
        // 如果查找的变量为参数的话，从参数中查找
        for param in inner.param_vars.iter() {
            if let Some(var_name) = param.name.clone() {
                if var_name == name {
                    return param.local_val.clone()
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

    pub fn add_stack_size(&self, size: isize) {
        let mut inner = self.inner.borrow_mut();
        if size > 0 {
            inner.stack_size += size as usize;
        }else{
            inner.stack_size -= -size as usize;
        }
    }

    /// push local variable into vec
    pub(crate) fn add_local_var(&self, var: Var) {
        let mut inner = self.inner.borrow_mut();
        inner.locals.push(var);
    }

    pub(crate) fn add_param_var(&self, var: Var) {
        let mut inner = self.inner.borrow_mut();
        inner.param_vars.push(var);
    }

    /// remove local variable in vec
    pub(crate) fn remove_local_var(&self, name: Name) {
        let mut inner = self.inner.borrow_mut();
        if let Some(index) = inner.locals.iter().position(|item| item.name == Some(name.clone())){
            inner.locals.remove(index);
        }
    }

    pub(crate) fn find_label(&self, name: Name) -> Option<Label> {
        let inner = self.inner.borrow();
        for item in inner.labels.iter() {
            if item.llvm_name == name {
                return Some(item.clone())
            }
        }
        None
    }
}