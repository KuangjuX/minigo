use super::{ Program, ProgInner };

impl ProgInner {
    pub(crate) fn allocate_physical_reg(&mut self) -> Option<PhysicalReg> {
        if let Some(reg) = self.regs.find_free_reg() {
            return Some(reg.clone())
        }
        None
    }

    pub(crate) fn free_physical_reg(&mut self, name: String) -> bool {
        self.regs.free_physical_name(name)
    }
}

#[derive(Debug, Clone)]
pub struct PhysicalReg {
    pub(crate) allocated: bool,
    pub(crate) index: usize,
    pub(crate) name: String
}

pub struct PhysicalRegs {
    regs: Vec<PhysicalReg>
}

impl PhysicalRegs {
    pub fn init() -> Self {
        let mut regs = vec![];
        for i in 1..11 {
            let reg = PhysicalReg{
                allocated: false,
                index: i,
                name: format!("s{}", i)
            };
            regs.push(reg);
        }
        Self{
            regs
        }
    }

    pub fn find_free_reg(&mut self) -> Option<&PhysicalReg> {
        for reg in self.regs.iter_mut() {
            if !reg.allocated {
                reg.allocated = true;
                return Some(reg)
            }
        }
        None
    }

    pub(crate) fn free_physical_name(&mut self, name: String) -> bool {
        for reg in self.regs.iter_mut() {
            if reg.name == name {
                reg.allocated = false;
                return true
            }
        }
        false
    }
}