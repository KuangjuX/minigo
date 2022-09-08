use crate::codegen::{Function, Program, PhysicalReg};

/// virtual reg in llvm_ir
#[derive(Debug)]
pub enum VirtualReg {
    Reg(RegVar),
    Stack(StackVar)
}

/// Store virtual reg into stack
#[derive(Debug)]
pub struct StackVar {
    /// stack address, find by offset of fp register
    pub addr: usize,
    /// stack pointer size
    pub size: usize
}

/// Store virtual reg into phyiscal reg
#[derive(Debug)]
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
    pub(crate) fn load_stack_var<'ctx>(&self, prog: &'ctx Program) -> Option<RegVar> {
        if let Some(physical_reg) = prog.allocate_physical_reg() {
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
}

impl VirtualReg {
    /// allocate register when creating virtual register,
    /// default create stack variable
    /// you can find this local variable by sd reg, addr(fp)
    pub fn naive_allocate_virt_reg(prog: &Program, func: &Function, size: usize) -> StackVar {
        let offset = func.stack_size();
        func.push_var(size);
        let stack_var = StackVar::new(offset, size);
        let asm = format!("    add sp, sp, -{}", size);
        prog.write_asm(asm);
        stack_var
    }

    pub fn optimized_allocate_virt_reg() -> Self {
        todo!()
    }
}