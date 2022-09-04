use llvm_ir::name::Name;


/// virtual reg in llvm_ir
pub enum VirtualReg {
    Reg(RegVar),
    Stack(StackVar),
    Uninit
}

/// Store virtual reg into stack
pub struct StackVar {
    /// virtual reg name
    pub name: Name,
    /// stack address, find by offset of fp register
    pub addr: usize
}

/// Store virtual reg into phyiscal reg
pub struct RegVar {
    /// name virtual reg name
    pub name: Name,
    /// physical register index
    pub id: usize
}

// impl VirtualReg {
//     pub fn allocate_reg() -> Result<usize, ()> {

//     }
// }