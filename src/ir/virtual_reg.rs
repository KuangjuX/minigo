use llvm_ir::name::Name;


/// virtual reg in llvm_ir
#[derive(Debug)]
pub enum VirtualReg {
    Reg(RegVar),
    Stack(StackVar)
}

use crate::codegen::{Function, Program};

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
    pub id: usize
}

impl StackVar {
    pub fn new(addr: usize, size: usize) -> Self {
        Self {
            addr,
            size
        }
    }
}

impl VirtualReg {
    /// allocate register when creating virtual register,
    /// default create stack variable
    /// you can find this local variable by sd reg, addr(fp)
    pub fn naive_allocate_virt_reg(prog: &Program, func: &mut Function, size: usize) -> Self {
        let stack_size = func.stack_size;
        func.stack_size += size;
        let stack_var = StackVar::new(stack_size, size);
        let asm = format!("    add sp, sp, -{}", size);
        prog.write_asm(asm);
        Self::Stack(stack_var)
    }

    pub fn optimized_allocate_virt_reg() -> Self {
        todo!()
    }
}