use crate::codegen::{Function, Program, Var, ProgInner};
use llvm_ir::name::Name;

/// virtual reg in llvm_ir
#[derive(Debug, Clone)]
pub enum VirtualReg {
    Reg(RegVar),
    Stack(StackVar)
}

/// Store virtual reg into stack
#[derive(Debug, Clone)]
pub struct StackVar {
    /// stack address, find by offset of fp register
    pub addr: usize,
    /// stack pointer size
    pub size: usize
}

/// Store virtual reg into phyiscal reg
#[derive(Debug, Clone)]
pub struct RegVar {
    /// physical register index
    pub id: usize,
    pub name: String
}



impl StackVar {
    pub fn new(addr: usize, size: usize) -> Self {
        Self {
            addr,
            size
        }
    }

    /// 将栈变量加载到 help register 中
    pub(crate) fn load_stack_var_1<'ctx>(&self, prog: &'ctx Program, prog_inner: &mut ProgInner) -> RegVar {
        // 使用 help register
        let help_reg = prog_inner.get_help_reg_1();
        let reg_name = help_reg.name.clone();
        let offset = self.addr;
        let asm = format!("    ld {}, -{}(fp)", reg_name, offset);
        prog.write_asm(asm);
        let reg_var = RegVar{ id: help_reg.index, name: help_reg.name };
        return reg_var
    }

    pub(crate) fn load_stack_var_2<'ctx>(&self, prog: &'ctx Program, prog_inner: &mut ProgInner) -> RegVar {
        // 使用 help register
        let help_reg = prog_inner.get_help_reg_2();
        let reg_name = help_reg.name.clone();
        let offset = self.addr;
        let asm = format!("    ld {}, -{}(fp)", reg_name, offset);
        prog.write_asm(asm);
        let reg_var = RegVar{ id: help_reg.index, name: help_reg.name };
        return reg_var
    }

    pub(crate) fn store_stack_var(&self, prog: &Program, reg_var: &RegVar) {
        let asm = format!("    sd {}, -{}(fp)", reg_var.name, self.addr);
        prog.write_asm(asm);
    }
}

impl RegVar {
    pub(crate) fn free_reg_var(&self, prog: &Program) {
        todo!()
    }
}

impl VirtualReg {
    /// 由于寄存器不够用，把变量放到栈上
    pub(crate) fn spill_virtual_var(prog: &Program, func: &Function, size: usize, name: Name) -> StackVar {
        Self::allocate_virt_stack_var(prog, func, size, name)
    }


    /// allocate stack register when creating virtual register,
    /// you can find this local variable by sd reg, addr(fp)
    /// TODO: push local var into function
    pub(crate) fn allocate_virt_stack_var(prog: &Program, func: &Function, size: usize, name: Name) -> StackVar {
        let offset = func.stack_size();
        func.push_var(size);
        let stack_var = StackVar::new(offset, size);
        let asm = format!("    addi sp, sp, -{}", size);
        prog.write_asm(asm);
        let mut var = Var::uninit();
        var.name = Some(name);
        var.local_val = Some(VirtualReg::Stack(stack_var.clone()));
        func.add_local_var(var);
        stack_var
    }

    /// allocate reg var when creating virtual reg
    /// push local var into function
    pub(crate) fn try_allocate_virt_reg_var(prog_inner: &mut ProgInner, func: &Function, name: Name) -> Option<RegVar> {
        if let Some(physical_reg) = prog_inner.allocate_physical_reg() {
            let reg_var = RegVar {
                id: physical_reg.index,
                name: physical_reg.name
            };
            let mut var = Var::uninit();
            var.name = Some(name);
            var.local_val = Some(VirtualReg::Reg(reg_var.clone()));
            func.add_local_var(var);
            return Some(reg_var)
        }
        // 当没有空闲的寄存器, 返回 None
        None
    }

    /// insert an used register into new virtual variable
    pub(crate) fn insert_virt_reg_var(prog_inner: &mut ProgInner, func: &Function, name: Name, reg_var: RegVar) {
        let mut var = Var::uninit();
        var.name = Some(name);
        var.local_val = Some(VirtualReg::Reg(reg_var.clone()));
        func.add_local_var(var);
    }

    pub fn optimized_allocate_virt_reg() -> Self {
        todo!()
    }
}