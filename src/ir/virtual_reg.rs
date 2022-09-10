use crate::codegen::{Function, Program, PhysicalReg, Var, ProgInner};
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

    /// Load a stack value into register
    /// return RegVar
    pub(crate) fn load_stack_var<'ctx>(&self, prog: &'ctx Program, prog_inner: &mut ProgInner) -> Option<RegVar> {
        if let Some(physical_reg) = prog_inner.allocate_physical_reg() {
            let reg_name = physical_reg.name.clone();
            let offset = self.addr;
            let asm = format!("    ld {}, -{}(fp)", reg_name, offset);
            prog.write_asm(asm);
            // return Some(physical_reg.clone())
            let reg_var = RegVar{ id: physical_reg.index, name: physical_reg.name };
            return Some(reg_var)
        }
        None
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
    /// allocate stack register when creating virtual register,
    /// you can find this local variable by sd reg, addr(fp)
    /// TODO: push local var into function
    pub(crate) fn allocate_virt_stack_var(prog: &Program, func: &Function, size: usize) -> StackVar {
        let offset = func.stack_size();
        func.push_var(size);
        let stack_var = StackVar::new(offset, size);
        let asm = format!("    add sp, sp, -{}", size);
        prog.write_asm(asm);
        stack_var
    }

    /// allocate reg var when creating virtual reg
    /// push local var into function
    pub(crate) fn allocate_virt_reg_var(prog_inner: &mut ProgInner, func: &Function, name: Name) -> Option<RegVar> {
        if let Some(physical_reg) = prog_inner.allocate_physical_reg() {
            let reg_var = RegVar {
                id: physical_reg.index,
                name: physical_reg.name
            };
            let mut var = Var::uninit();
            var.name = Some(name);
            var.local_val = Some(VirtualReg::Reg(reg_var.clone()));
            // println!("var: {:?}", var);
            func.add_local_var(var);
            return Some(reg_var)
        }
        None
    }

    pub fn optimized_allocate_virt_reg() -> Self {
        todo!()
    }
}