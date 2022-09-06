
pub struct PhysicalReg {
    allocated: bool,
    index: usize,
    name: String
}

pub struct PhysicalRegs {
    regs: [PhysicalReg; 32]
}

impl PhysicalRegs {
    pub fn find_free_reg(&mut self) -> Option<usize> {
        todo!()
    }
}